use super::*;

struct PrimaryPart {
    range: [usize; 2],
    width: usize,
    height: usize,
    rgb: Vec<u8>,
    space_before: bool,
}

fn primary_partitions(
    rgb: &[u8],
    width: usize,
    height: usize,
    iw: usize,
    ih: usize,
) -> Result<[Vec<PrimaryPart>; 2], String> {
    let (column, _) = prepare_otsu(rgb, width, height).ok_or("invalid or uniform primary RGB")?;
    let split = |divisor| -> Result<Vec<PrimaryPart>, String> {
        let chunks = crate::pipeline::line_chunks::split_supported_vertical_column_for_input(
            &column.support,
            column.width,
            column.height,
            220,
            iw,
            ih,
            divisor,
        )?;
        chunks
            .iter()
            .enumerate()
            .map(|(index, chunk)| {
                let crop = crate::pipeline::line_chunks::crop_line_chunk_rgb_u8(
                    &column.original,
                    column.width,
                    column.height,
                    &chunks,
                    index,
                )
                .map_err(|error| error.to_string())?;
                Ok(PrimaryPart {
                    range: [chunk.x0, chunk.x1],
                    width: crop.width,
                    height: crop.height,
                    rgb: crop.data,
                    space_before: chunk.space_before,
                })
            })
            .collect()
    };
    Ok([split(1)?, split(2)?])
}

#[test]
#[ignore = "diagnostic only: explicit frozen primary RGB manifest and new output directory; NN0"]
fn record_all_primary_partitions_without_nn() {
    use std::io::{Read, Write};
    let Some(path) = std::env::var_os("NDLOCR_PRIMARY_PARTITION_MANIFEST") else {
        return;
    };
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let output =
        std::path::PathBuf::from(std::env::var_os("NDLOCR_PRIMARY_PARTITION_OUTPUT").unwrap());
    std::fs::create_dir(&output).unwrap();
    let items = manifest["items"].as_array().unwrap();
    assert!(!items.is_empty());
    let mut archive = zip::ZipWriter::new(std::fs::File::create(output.join("parts.zip")).unwrap());
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut records = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for item in items {
        let id = item["id"].as_str().unwrap();
        assert!(id.bytes().all(|b| b.is_ascii_hexdigit()));
        assert!(seen.insert(id));
        let mut input =
            zip::ZipArchive::new(std::fs::File::open(item["archive"].as_str().unwrap()).unwrap())
                .unwrap();
        let mut rgb = Vec::new();
        input
            .by_name(item["member"].as_str().unwrap())
            .unwrap()
            .read_to_end(&mut rgb)
            .unwrap();
        let original = rgb.clone();
        let width = item["width"].as_u64().unwrap() as usize;
        let height = item["height"].as_u64().unwrap() as usize;
        let iw = item["input"][0].as_u64().unwrap() as usize;
        let ih = item["input"][1].as_u64().unwrap() as usize;
        let mut record = serde_json::json!({"id":id,"width":width,"height":height,"input":[iw,ih],"routes":null,"error":null});
        match primary_partitions(&rgb, width, height, iw, ih) {
            Ok(routes) => {
                let mut saved = Vec::new();
                for (route, parts) in routes.into_iter().enumerate() {
                    let mut rows = Vec::new();
                    for (index, part) in parts.into_iter().enumerate() {
                        let name = format!("{id}-{route}-{index}.rgb");
                        archive.start_file(&name, options).unwrap();
                        archive.write_all(&part.rgb).unwrap();
                        rows.push(serde_json::json!({"range":part.range,"width":part.width,"height":part.height,"space_before":part.space_before,"file":name}));
                    }
                    saved.push(rows);
                }
                record["routes"] = serde_json::json!(saved);
            }
            Err(error) => record["error"] = serde_json::json!(error),
        }
        assert_eq!(rgb, original);
        records.push(record);
    }
    archive.finish().unwrap();
    std::fs::write(
        output.join("records.json"),
        serde_json::to_vec_pretty(&records).unwrap(),
    )
    .unwrap();
    println!(
        "{} primary model/RGB inputs; all geometry outcomes preserved; NN0",
        records.len()
    );
}

#[test]
fn supported_partitions_keep_the_original_pixels_and_never_recolor_the_paper() {
    let (width, height) = (24, 43 * 22 + 4);
    let mut rgb: Vec<u8> = (0..width * height).flat_map(|_| [240, 220, 170]).collect();
    for glyph in 0..43 {
        for y in 3 + glyph * 22..17 + glyph * 22 {
            for x in 5..18 {
                rgb[(y * width + x) * 3..(y * width + x) * 3 + 3].copy_from_slice(&[0; 3]);
            }
        }
    }
    let before = rgb.clone();
    let routes = primary_partitions(&rgb, width, height, 256, 24).unwrap();
    assert!(routes[0].len() >= 2);
    assert!(routes[1].len() > routes[0].len());
    for route in routes {
        for part in route {
            assert_eq!(
                (part.width, part.height),
                (part.range[1] - part.range[0], width)
            );
            assert!(!part.space_before);
            for y in 0..part.height {
                for x in 0..part.width {
                    let a = (y * part.width + x) * 3;
                    let b = ((part.range[0] + x) * width + (width - 1 - y)) * 3;
                    assert_eq!(&part.rgb[a..a + 3], &rgb[b..b + 3]);
                }
            }
        }
    }
    assert_eq!(rgb, before);
}

#[test]
fn malformed_or_uniform_inputs_report_failure_without_a_partial_geometry() {
    for (rgb, w, h) in [(vec![], 0, 1), (vec![0; 3], 1, 2), (vec![240; 72], 2, 12)] {
        assert!(primary_partitions(&rgb, w, h, 256, 24).is_err());
    }
}
