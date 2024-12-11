use std::io::Write;
use std::str::FromStr;
use std::{fs, io::Read, path::PathBuf};

use anyhow::anyhow;
use aws_sdk_bedrockruntime::primitives::Blob;
use aws_sdk_bedrockruntime::types::{
    ContentBlock, ImageBlock, ImageFormat, ImageSource, InferenceConfiguration, SystemContentBlock,
};

use indicatif::{ProgressBar, ProgressStyle};
use quick_xml::se;
use serde::Serialize;

use crate::models::check_model_features;
use crate::models::converse::call_converse;
use crate::models::ModelFeatures;
use crate::utils::BedrustConfig;

#[derive(Debug, Serialize)]
pub struct Image {
    pub path: PathBuf,
    #[serde(skip_serializing)]
    pub extension: String,
    // FIX: Think about setting the base64 as optional
    #[serde(skip_serializing)]
    //pub base64: String,
    pub base64: Vec<u8>,
    pub caption: Option<String>,
}

pub enum OutputFormat {
    Json,
    Xml,
}

impl Image {
    pub fn new(p: &PathBuf) -> Result<Self, anyhow::Error> {
        let extension = &p
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("The file provided has no extension, this is an issue."))?;
        let img_base64 = load_image(p)?;

        // FIX: Clone happens - see if we should remove that
        let image = Self {
            path: p.clone(),
            extension: extension.to_string(),
            base64: img_base64,
            caption: None,
        };
        Ok(image)
    }
}

// This function wraps a bunch of other steps in order to capiton an image (check for model
// capabilities and such).
// This is for the sole reason of moving this out of the main.rs function
pub async fn caption_process(
    model_id: &str,
    bedrock_client: &aws_sdk_bedrock::Client,
    bedrockruntime_client: &aws_sdk_bedrockruntime::Client,
    images_path: Option<PathBuf>,
    bedrust_config: &BedrustConfig,
    xml: bool,
) -> Result<(), anyhow::Error> {
    match check_model_features(model_id, bedrock_client, ModelFeatures::Images).await {
        Ok(b) => {
            match b {
                true => {
                    println!("----------------------------------------");
                    println!("🖼️ | Image captioner running.");
                    let path = images_path.ok_or_else(|| anyhow!("No path specified"))?;
                    println!("⌛ | Processing images in: {:?}", &path);
                    let files = list_files_in_path_by_extension(
                        path,
                        bedrust_config.supported_images.clone(),
                    )?;
                    println!("🔎 | Found {:?} images in path.", &files.len());

                    let mut images: Vec<Image> = Vec::new();
                    for file in &files {
                        images.push(Image::new(file)?);
                    }

                    caption_image(
                        &mut images,
                        model_id,
                        &bedrust_config.caption_prompt,
                        bedrockruntime_client,
                        bedrock_client,
                    )
                    .await?;

                    // NOTE: This is parsing the `-x` argument and then writing or not, an XML file
                    // Thanks StellyUK <3

                    // FIX: This whole if else statement does not look nice.
                    // i feel it can be better. As doing the whole logic
                    // behind an expression seems ... weird
                    let outfile = if xml {
                        let outfile = "captions.xml";
                        write_captions(images, OutputFormat::Xml, outfile)?;
                        outfile
                    } else {
                        let outfile = "captions.json";
                        write_captions(images, OutputFormat::Json, outfile)?;
                        outfile
                    };
                    println!(
                        "✅ | Captioning complete, find the generated captions in `{}`",
                        outfile
                    );
                    println!("----------------------------------------");
                }
                false => {
                    eprintln!("The current model selected does not support Images. Please consider using one that does.")
                }
            }
        }
        Err(e) => eprintln!("Unable to determine model features: {}", e),
    };

    Ok(())
}

pub fn list_files_in_path_by_extension(
    p: PathBuf,
    ext: Vec<String>,
) -> Result<Vec<PathBuf>, anyhow::Error> {
    let entries = fs::read_dir(p)?;
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("There was no files with extensions in this directory"))?;
        if ext.contains(&extension.to_string()) {
            files.push(path);
        }
    }
    Ok(files)
}

//pub fn load_image(p: &PathBuf) -> Result<String, anyhow::Error> {
pub fn load_image(p: &PathBuf) -> Result<Vec<u8>, anyhow::Error> {
    let mut file = fs::File::open(p)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

pub async fn caption_image(
    i: &mut Vec<crate::captioner::Image>,
    model: &str,
    prompt: &str,
    runtime_client: &aws_sdk_bedrockruntime::Client,
    _bedrock_client: &aws_sdk_bedrock::Client,
) -> Result<(), anyhow::Error> {
    // Needs to be hardcoded for images
    let inference_parameters: InferenceConfiguration = InferenceConfiguration::builder()
        .max_tokens(2048)
        .top_p(0.8)
        .temperature(0.5)
        .build();

    // FIX: Remove the clone
    let system_prompt = Some(vec![SystemContentBlock::Text(prompt.to_owned())]);

    // progress bar shenanigans
    let progress_bar = ProgressBar::new(i.len().try_into()?);
    progress_bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{wide_bar:.cyan/blue}] {msg} ({pos}/{len})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    progress_bar.enable_steady_tick(std::time::Duration::from_millis(100));

    for image in i {
        let message = image.path.as_path().display().to_string();

        //let imagesrc: ImageSource = ImageSource::Bytes(image.base64.to_blob());
        let imagesrc: ImageSource = ImageSource::Bytes(
            // FIX: Try to remove the clone
            Blob::new(image.base64.clone()),
        );

        let image_block = ImageBlock::builder()
            .source(imagesrc)
            .format(ImageFormat::from_str(image.extension.as_str())?)
            .build()?;

        let content = ContentBlock::Image(image_block);
        progress_bar.set_message(message);
        // let caption = ask_bedrock(
        //     prompt,
        //     Some(image),
        //     model,
        //     RunType::Captioning,
        //     runtime_client,
        //     bedrock_client,
        // )
        // .await?;
        let caption = call_converse(
            runtime_client,
            model.to_string(),
            // FIX: Avoid the clone
            inference_parameters.clone(),
            content,
            // FIX: Avoid the clone
            system_prompt.clone(),
            false,
        )
        .await?;
        progress_bar.inc(1);
        image.caption = Some(caption);
    }
    progress_bar.finish();

    Ok(())
}

pub fn write_captions(
    i: Vec<crate::captioner::Image>,
    format: OutputFormat,
    filename: &str,
) -> Result<(), anyhow::Error> {
    match format {
        OutputFormat::Json => {
            let mut json_file = std::fs::File::create(filename).expect("Failed to create file");
            let json_serialized =
                serde_json::to_string_pretty(&i).expect("Failed to serialize data");
            json_file
                .write_all(json_serialized.as_bytes())
                .expect("Failed to write to file");
        }
        OutputFormat::Xml => {
            let mut xml_file = std::fs::File::create(filename).expect("Failed to create file");
            let xmled = se::to_string_with_root("captions", &i).expect("Failed to convert to XML");
            xml_file
                .write_all(xmled.as_bytes())
                .expect("Failed to write to file");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::load_bedrust_config;
    use base64::{engine::general_purpose, Engine as _};
    use image::{Rgb, RgbImage};
    use rand::distributions::{Alphanumeric, DistString};

    // list all files in the directory
    #[test]
    fn list_all_images() {
        // prep the test by creating a know directory
        // generate random string for dir name
        let dir_prefix = Alphanumeric.sample_string(&mut rand::thread_rng(), 5);
        let dir_path = format!("/tmp/{}-bedrusttest", dir_prefix);
        // create the dir
        // TODO: handle issues creating the path
        fs::create_dir_all(&dir_path).unwrap();
        // creating files:
        let file1_path = format!("{}/file1.jpeg", &dir_path);
        let file2_path = format!("{}/123456789.md5", &dir_path);
        let file3_path = format!("{}/alanford.jpg", &dir_path);
        let file4_path = format!("{}/bobrok.png", &dir_path);
        let file5_path = format!("{}/superhik.bmp", &dir_path);
        // TODO: handle issues creating the path
        fs::File::create(&file1_path).unwrap();
        fs::File::create(file2_path).unwrap();
        fs::File::create(&file3_path).unwrap();
        fs::File::create(&file4_path).unwrap();
        fs::File::create(&file5_path).unwrap();

        // load supported file extensions
        let config = load_bedrust_config().unwrap();

        let list =
            list_files_in_path_by_extension(PathBuf::from(dir_path), config.supported_images);
        let expected_vec = vec![
            PathBuf::from(&file1_path),
            PathBuf::from(&file3_path),
            PathBuf::from(&file4_path),
            PathBuf::from(&file5_path),
        ];
        assert_eq!(expected_vec, list.unwrap());
    }
    #[test]
    fn load_image_from_disk() {
        let image_path = PathBuf::from("/tmp/bedrust_test_image.jpeg");
        // Generate an image
        let mut img = RgbImage::new(32, 32);
        for x in 15..=17 {
            for y in 8..24 {
                img.put_pixel(x, y, Rgb([255, 0, 0]));
                img.put_pixel(y, x, Rgb([255, 0, 0]));
            }
        }
        img.save(&image_path).unwrap();

        // Load the generated image from disk
        let test_image = load_image(&PathBuf::from(&image_path)).unwrap();

        // This is just raw base64 of the generated image from above
        let expected_image_base64 = "/9j/4AAQSkZJRgABAgAAAQABAAD/wAARCAAgACADAREAAhEBAxEB/9sAQwAIBgYHBgUIBwcHCQkICgwUDQwLCwwZEhMPFB0aHx4dGhwcICQuJyAiLCMcHCg3KSwwMTQ0NB8nOT04MjwuMzQy/9sAQwEJCQkMCwwYDQ0YMiEcITIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIy/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/8QAHwEAAwEBAQEBAQEBAQAAAAAAAAECAwQFBgcICQoL/8QAtREAAgECBAQDBAcFBAQAAQJ3AAECAxEEBSExBhJBUQdhcRMiMoEIFEKRobHBCSMzUvAVYnLRChYkNOEl8RcYGRomJygpKjU2Nzg5OkNERUZHSElKU1RVVldYWVpjZGVmZ2hpanN0dXZ3eHl6goOEhYaHiImKkpOUlZaXmJmaoqOkpaanqKmqsrO0tba3uLm6wsPExcbHyMnK0tPU1dbX2Nna4uPk5ebn6Onq8vP09fb3+Pn6/9oADAMBAAIRAxEAPwD5/oAKACgAoAKANKwsLxNRtWa0nVVmQkmMgAZHtXNWrU3TklJbPqe1l2XYyGMpSlSkkpR+y+68ixrlldzazcPHazOh24ZYyQflFZ4SrTjRScl9/mdvEGAxVXMak6dKTTtqk2vhXkYtdp8wFAG1Za5qM1/bxvcZR5VVhsXkE/SuKrhKMacml0fc+nwHEGY1cVSpzqXTkk9I7NryJ9Y1i/tdVmhhn2xrtwNin+EHuKzw2GpTpKUlr8+51Z3nePw2PqUaNS0VaysuyfVHPV6J8cFABQAUAFAH/9k=";

        // Convert the Vec<u8> to base64 string for comparison
        let actual_image_base64 = general_purpose::STANDARD.encode(&test_image);

        assert_eq!(actual_image_base64, expected_image_base64);

        // Clean up: remove the test image file
        fs::remove_file(image_path).unwrap();
    }
}
