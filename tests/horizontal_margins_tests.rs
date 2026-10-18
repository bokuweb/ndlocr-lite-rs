use ndlocr_lite_rs::pipeline::horizontal_margins::normalize_horizontal_margins as normalize;
use proptest::prelude::*;
/// 非白が左右端にある入力でも、全行・内部空白・微弱RGBを保持する。
#[test]
fn retains_near_white_internal_spaces_and_full_height() {
    let rgb = [
        255, 254, 255, 255, 255, 255, 2, 3, 4, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    ];
    let out = normalize(&rgb, 3, 2, 1).unwrap();
    assert_eq!((out.source_x, out.content_width, out.output_x), (0, 3, 1));
    assert_eq!((out.image.width, out.image.height), (5, 2));
    for y in 0..2 {
        assert_eq!(
            &out.image.data[(y * 5 + 1) * 3..(y * 5 + 4) * 3],
            &rgb[y * 9..(y + 1) * 9]
        );
    }
}
/// 白紙はmarginに依らず非空canvasを維持する。
#[test]
fn blank_is_identity() {
    let out = normalize(&[255; 18], 3, 2, 2).unwrap();
    assert_eq!((out.image.width, out.image.height), (3, 2));
    assert_eq!((out.source_x, out.content_width, out.output_x), (0, 3, 0));
    assert_eq!(out.image.data, vec![255; 18]);
}
/// 不正寸法・長さ・過大margin・積overflowを拒否する。
#[test]
fn rejects_invalid_inputs() {
    for (rgb, w, h, m) in [
        (&[][..], 0, 1, 0),
        (&[][..], 1, 0, 0),
        (&[0][..], 1, 1, 0),
        (&[255; 3][..], 1, 1, 2),
        (&[][..], usize::MAX, 2, 0),
    ] {
        assert!(normalize(rgb, w, h, m).is_err(), "{w} {h} {m}");
    }
}
proptest! {
 /// 全RGBの出典が一致し、追加白列に依らず同一canvasとなる。
 #[test]
 fn padding_invariant_and_preserves_every_nonwhite(
  w in 1usize..30,h in 1usize..12,left in 0usize..8,right in 0usize..8,
  bytes in prop::collection::vec(any::<u8>(),1..1081),margin in 0usize..2
 ) {
  let margin=margin.min(w);
  let rgb:Vec<u8>=(0..w*h*3).map(|i|bytes[i%bytes.len()]).collect();
  let out=normalize(&rgb,w,h,margin).unwrap();
  let mut padded=vec![255;(w+left+right)*h*3];
  for y in 0..h {padded[(y*(w+left+right)+left)*3..(y*(w+left+right)+left+w)*3].copy_from_slice(&rgb[y*w*3..(y+1)*w*3]);}
  let other=normalize(&padded,w+left+right,h,margin).unwrap();
  if rgb.chunks_exact(3).any(|p|p!=[255;3]) {
   prop_assert_eq!(&out.image,&other.image);
   let xs:Vec<usize>=rgb.chunks_exact(3).enumerate().filter(|(_,p)|*p!=[255;3]).map(|(i,_)|i%w).collect();
   let a=*xs.iter().min().unwrap();let b=*xs.iter().max().unwrap()+1;
   prop_assert_eq!((out.source_x,out.content_width,out.output_x),(a,b-a,margin));
   prop_assert_eq!(out.image.width,b-a+2*margin);
   for y in 0..h {for x in 0..out.image.width {
    let actual=&out.image.data[(y*out.image.width+x)*3..(y*out.image.width+x+1)*3];
    if (margin..margin+b-a).contains(&x) {
     let sx=x-margin+a;
     prop_assert_eq!(actual,&rgb[(y*w+sx)*3..(y*w+sx+1)*3]);
    } else {prop_assert_eq!(actual,[255;3]);}
   }}
  }
 }
}
