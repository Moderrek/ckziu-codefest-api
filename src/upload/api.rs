use futures::{StreamExt, TryStreamExt};
use multipart::FormData;
use uuid::Uuid;
use warp::{multipart, reject, Reply};
use warp::Buf;
use warp::multipart::Part;

use crate::{error, WebResult};

async fn save_part_to_file(path: &str, part: Part) -> Result<(), tokio::io::Error> {
  let data = part.stream().try_fold(Vec::new(), |mut acc, buf| async move {
    acc.extend_from_slice(buf.chunk());
    Ok(acc)
  }).await.expect("Folding error");
  tokio::fs::write(path, data).await
}

pub async fn upload_profile_picture(user_uid: Option<Uuid>, form: FormData) -> WebResult<impl Reply> {
  // Reject unauthorized
  if user_uid.is_none() {
    return Err(reject::custom(error::Error::Unauthorized));
  }
  // Collect stream
  tokio::task::spawn(async move {
    let mut parts = form.into_stream();

    while let Ok(p) = parts.next().await.unwrap() {
      let filepath = format!("uploads/{}", Uuid::new_v4());

      tokio::fs::create_dir_all("uploads").await.unwrap();

      save_part_to_file(&filepath, p).await.unwrap()
    }
  });

  Ok("Uploaded!")
}