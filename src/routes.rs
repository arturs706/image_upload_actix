use actix_web::{post, web, HttpResponse, HttpRequest};
use actix_multipart::Multipart;
use futures_util::TryStreamExt as _;
use mime::{Mime, IMAGE_PNG, IMAGE_JPEG, IMAGE_GIF, IMAGE_BMP};
use tokio::fs;
use tokio::io::AsyncWriteExt as _;
use image::{DynamicImage, imageops::FilterType};
use uuid::Uuid;

#[post("/api/v1/properties/upload")]
pub async fn upload(mut payload: Multipart, req: HttpRequest) -> HttpResponse {
    let content_length: usize = match req.headers().get("content-length") {
        Some(header_value) => header_value.to_str().unwrap_or("0").parse().unwrap(),
        None => "0".parse().unwrap(),
    };
    println!("content_length: {:#?}", content_length);

    let max_file_count: usize = 10;
    let max_file_size: usize = 10_000_000;
    let legal_filetypes: [Mime; 4] = [IMAGE_PNG, IMAGE_JPEG, IMAGE_GIF, IMAGE_BMP];
    let mut current_count: usize = 0;
    let dir: &str = "./upload/";

    if content_length > max_file_size {
        return HttpResponse::BadRequest().into();
    }

    while let Ok(Some(mut field)) = payload.try_next().await {
        if current_count >= max_file_count {
            break;
        }
        let filetype: Option<&Mime> = field.content_type();
        if filetype.is_none() {
            continue;
        }
        if !legal_filetypes.contains(&filetype.unwrap()) {
            continue;
        }
        let destination: String = format!(
            "{}{}-{}",
            dir,
            Uuid::new_v4(),
            field.content_disposition().get_filename().unwrap()
        );

        let mut saved_file: fs::File = fs::File::create(&destination).await.unwrap();
        while let Ok(Some(chunk)) = field.try_next().await {
            let _ = saved_file.write_all(&chunk).await.unwrap();
        }

        tokio::spawn(handle_image_processing(destination.clone()));
        current_count += 1;
    }

    HttpResponse::Ok().body(format!("{} files uploaded", current_count))
}

async fn handle_image_processing(destination: String) {
    let uploaded_img: DynamicImage = image::open(&destination).unwrap();
    let _ = fs::remove_file(&destination).await.unwrap();
    uploaded_img
        .resize_exact(1920, 1080, FilterType::Gaussian)
        .save(format!("{}.avif", destination)).unwrap();
}
