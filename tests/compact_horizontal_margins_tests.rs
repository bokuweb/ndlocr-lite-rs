use ndlocr_lite_rs::pipeline::horizontal_margins::compact_horizontal_margins as compact;
use proptest::prelude::*;
/// 前景が端に達する密な入力を拡幅せず全RGBと位置を保つ。
#[test]
fn dense_input_is_exact_identity() {
    let rgb = [
        0, 1, 2, 255, 255, 255, 254, 255, 255, 255, 255, 255, 3, 4, 5, 255, 255, 255,
    ];
    let out = compact(&rgb, 3, 2, 1).unwrap();
    assert_eq!((out.image.width, out.image.height), (3, 2));
    assert_eq!(out.image.data, rgb);
    assert_eq!((out.source_x, out.content_width, out.output_x), (0, 3, 0));
}
/// 同幅でも非対称余白の位置移動を避ける。
#[test]
fn equal_width_asymmetric_margin_is_identity() {
    let rgb = [255, 255, 255, 255, 255, 255, 0, 1, 2];
    let out = compact(&rgb, 3, 1, 1).unwrap();
    assert_eq!(out.image.data, rgb);
    assert_eq!((out.source_x, out.content_width, out.output_x), (0, 3, 0));
}
/// 白紙と不正入力は既存APIと同じ契約を維持する。
#[test]
fn blank_and_invalid_inputs() {
    let out = compact(&[255; 18], 3, 2, 2).unwrap();
    assert_eq!(out.image.data, [255; 18]);
    assert_eq!((out.source_x, out.content_width, out.output_x), (0, 3, 0));
    for (rgb, w, h, m) in [
        (&[][..], 0, 1, 0),
        (&[][..], 1, 0, 0),
        (&[0][..], 1, 1, 0),
        (&[255; 3][..], 1, 1, 2),
        (&[][..], usize::MAX, 2, 0),
    ] {
        assert!(compact(rgb, w, h, m).is_err());
    }
}
proptest! {
 /// 独立x区間oracleで縮小条件・全RGB保存を検査し、縮小時の白追加不変も確認する。
 #[test]
 fn contracts_only_and_keeps_pixel_mapping(
  w in 1usize..30,h in 1usize..12,left in 0usize..8,right in 0usize..8,
  bytes in prop::collection::vec(any::<u8>(),1..1081),m in 0usize..8
 ) {
  let margin=m.min(w);
  let mut rgb=vec![255;w*h*3];
  for y in 0..h {for x in 0..w {if x>=left.min(w) && x<w.saturating_sub(right) {
   for c in 0..3 {rgb[(y*w+x)*3+c]=bytes[(y*w*3+x*3+c)%bytes.len()];}
  }}}
  let out=compact(&rgb,w,h,margin).unwrap();
  let xs:Vec<_>=rgb.chunks_exact(3).enumerate().filter(|(_,p)|*p!=[255;3]).map(|(i,_)|i%w).collect();
  let interval=xs.iter().min().zip(xs.iter().max()).map(|(&a,&b)|(a,b+1));
  let shrinks=interval.is_some_and(|(a,b)|b-a+2*margin<w);
  prop_assert!(out.image.width<=w);
  prop_assert_eq!(out.image.height,h);
  if let Some((a,b))=interval.filter(|_|shrinks) {
   prop_assert_eq!((out.source_x,out.content_width,out.output_x),(a,b-a,margin));
   prop_assert_eq!(out.image.width,b-a+2*margin);
   for y in 0..h {for x in 0..out.image.width {
    let actual=&out.image.data[(y*out.image.width+x)*3..(y*out.image.width+x+1)*3];
    if (margin..margin+b-a).contains(&x) {
     let sx=x-margin+a;
     prop_assert_eq!(actual,&rgb[(y*w+sx)*3..(y*w+sx+1)*3]);
    } else {prop_assert_eq!(actual,[255;3]);}
   }}
   let pw=w+left+right;
   let mut padded=vec![255;pw*h*3];
   for y in 0..h {padded[(y*pw+left)*3..(y*pw+left+w)*3].copy_from_slice(&rgb[y*w*3..(y+1)*w*3]);}
   let other=compact(&padded,pw,h,margin).unwrap();
   prop_assert_eq!(out.image,other.image);
  } else {
   prop_assert_eq!(out.image.data,rgb);
   prop_assert_eq!((out.source_x,out.content_width,out.output_x),(0,w,0));
  }
 }
}
