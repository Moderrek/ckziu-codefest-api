use lettre::{Message, SmtpTransport, Transport};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;

pub fn send_otp_code(code: String) {

  let email = Message::builder()
    .from("CKZiU CodeFest <noreply@ckziucodefest.pl>".parse().unwrap())
    .to("tymonek12345@gmail.com".parse().unwrap())
    .subject("[CODEFEST] Twój jednorazowy kod autoryzacyjny")
    .header(ContentType::TEXT_HTML)
    .body(format!("Twój jednorazowy kod autoryzacyjny to: `{}`", code))
    .unwrap();

  let mailer: SmtpTransport = SmtpTransport::relay("ssl0.ovh.net").expect("Failed to connect to SMTP")
    .credentials(Credentials::new("noreply@ckziucodefest.pl".into(), "FJWuSbmWKeRpePqX".into()))
    .build();

  let result = mailer.send(&email);
  println!("{:?}", result);
}