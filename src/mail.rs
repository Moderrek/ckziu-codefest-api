use lettre::{Message, SmtpTransport, Transport};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;

pub fn send_otp_code(code: String, receiver: String) {
  let email = Message::builder()
    .from("CKZiU CodeFest <noreply@ckziucodefest.pl>".parse().unwrap())
    .to(receiver.parse().unwrap())
    .subject("[CODEFEST] Twój jednorazowy kod autoryzacyjny")
    .header(ContentType::TEXT_HTML)
    .body(r#"
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office" lang="en">
<head>
<title></title>
<meta charset="UTF-8" />
<meta http-equiv="Content-Type" content="text/html; charset=UTF-8" />
<!--[if !mso]><!-->
<meta http-equiv="X-UA-Compatible" content="IE=edge" />
<!--<![endif]-->
<meta name="x-apple-disable-message-reformatting" content="" />
<meta content="target-densitydpi=device-dpi" name="viewport" />
<meta content="true" name="HandheldFriendly" />
<meta content="width=device-width" name="viewport" />
<meta name="format-detection" content="telephone=no, date=no, address=no, email=no, url=no" />
<style type="text/css">
table {
border-collapse: separate;
table-layout: fixed;
mso-table-lspace: 0pt;
mso-table-rspace: 0pt
}
table td {
border-collapse: collapse
}
.ExternalClass {
width: 100%
}
.ExternalClass,
.ExternalClass p,
.ExternalClass span,
.ExternalClass font,
.ExternalClass td,
.ExternalClass div {
line-height: 100%
}
body, a, li, p, h1, h2, h3 {
-ms-text-size-adjust: 100%;
-webkit-text-size-adjust: 100%;
}
html {
-webkit-text-size-adjust: none !important
}
body, #innerTable {
-webkit-font-smoothing: antialiased;
-moz-osx-font-smoothing: grayscale
}
#innerTable img+div {
display: none;
display: none !important
}
img {
Margin: 0;
padding: 0;
-ms-interpolation-mode: bicubic
}
h1, h2, h3, p, a {
line-height: 1;
overflow-wrap: normal;
white-space: normal;
word-break: break-word
}
a {
text-decoration: none
}
h1, h2, h3, p {
min-width: 100%!important;
width: 100%!important;
max-width: 100%!important;
display: inline-block!important;
border: 0;
padding: 0;
margin: 0
}
a[x-apple-data-detectors] {
color: inherit !important;
text-decoration: none !important;
font-size: inherit !important;
font-family: inherit !important;
font-weight: inherit !important;
line-height: inherit !important
}
u + #body a {
color: inherit;
text-decoration: none;
font-size: inherit;
font-family: inherit;
font-weight: inherit;
line-height: inherit;
}
a[href^="mailto"],
a[href^="tel"],
a[href^="sms"] {
color: inherit;
text-decoration: none
}
img,p{margin:0;Margin:0;font-family:Lato,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:22px;font-weight:400;font-style:normal;font-size:16px;text-decoration:none;text-transform:none;letter-spacing:0;direction:ltr;color:#333;text-align:left;mso-line-height-rule:exactly;mso-text-raise:2px}h1{margin:0;Margin:0;font-family:Roboto,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:34px;font-weight:400;font-style:normal;font-size:28px;text-decoration:none;text-transform:none;letter-spacing:0;direction:ltr;color:#333;text-align:left;mso-line-height-rule:exactly;mso-text-raise:2px}h2{margin:0;Margin:0;font-family:Lato,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:30px;font-weight:400;font-style:normal;font-size:24px;text-decoration:none;text-transform:none;letter-spacing:0;direction:ltr;color:#333;text-align:left;mso-line-height-rule:exactly;mso-text-raise:2px}h3{margin:0;Margin:0;font-family:Lato,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:26px;font-weight:400;font-style:normal;font-size:20px;text-decoration:none;text-transform:none;letter-spacing:0;direction:ltr;color:#333;text-align:left;mso-line-height-rule:exactly;mso-text-raise:2px}
</style>
<style type="text/css">
@media (min-width: 481px) {
.hd { display: none!important }
}
</style>
<style type="text/css">
@media (max-width: 480px) {
.hm { display: none!important }
}
</style>
<style type="text/css">
@media (min-width: 481px) {
h1,img,p{margin:0;Margin:0}img,p{font-family:Lato,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:22px;font-weight:400;font-style:normal;font-size:16px;text-decoration:none;text-transform:none;letter-spacing:0;direction:ltr;color:#333;text-align:left;mso-line-height-rule:exactly;mso-text-raise:2px}h1{font-family:Roboto,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:34px;font-weight:400;font-style:normal;font-size:28px;text-decoration:none;text-transform:none;letter-spacing:0;direction:ltr;color:#333;text-align:left;mso-line-height-rule:exactly;mso-text-raise:2px}h2,h3{margin:0;Margin:0;font-family:Lato,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;font-weight:400;font-style:normal;text-decoration:none;text-transform:none;letter-spacing:0;direction:ltr;color:#333;text-align:left;mso-line-height-rule:exactly;mso-text-raise:2px}h2{line-height:30px;font-size:24px}h3{line-height:26px;font-size:20px}.t23,.t36{max-width:600px!important}
}
</style>
<style type="text/css" media="screen and (min-width:481px)">.moz-text-html img,.moz-text-html p{margin:0;Margin:0;font-family:Lato,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:22px;font-weight:400;font-style:normal;font-size:16px;text-decoration:none;text-transform:none;letter-spacing:0;direction:ltr;color:#333;text-align:left;mso-line-height-rule:exactly;mso-text-raise:2px}.moz-text-html h1{margin:0;Margin:0;font-family:Roboto,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:34px;font-weight:400;font-style:normal;font-size:28px;text-decoration:none;text-transform:none;letter-spacing:0;direction:ltr;color:#333;text-align:left;mso-line-height-rule:exactly;mso-text-raise:2px}.moz-text-html h2{margin:0;Margin:0;font-family:Lato,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:30px;font-weight:400;font-style:normal;font-size:24px;text-decoration:none;text-transform:none;letter-spacing:0;direction:ltr;color:#333;text-align:left;mso-line-height-rule:exactly;mso-text-raise:2px}.moz-text-html h3{margin:0;Margin:0;font-family:Lato,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:26px;font-weight:400;font-style:normal;font-size:20px;text-decoration:none;text-transform:none;letter-spacing:0;direction:ltr;color:#333;text-align:left;mso-line-height-rule:exactly;mso-text-raise:2px}.moz-text-html .t23,.moz-text-html .t36{max-width:600px!important}</style>
<!--[if !mso]><!-->
<link href="https://fonts.googleapis.com/css2?family=Fira+Sans:wght@400;500;600;700&amp;display=swap" rel="stylesheet" type="text/css" />
<!--<![endif]-->
<!--[if mso]>
<style type="text/css">
img,p{margin:0;Margin:0;font-family:Lato,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:22px;font-weight:400;font-style:normal;font-size:16px;text-decoration:none;text-transform:none;letter-spacing:0;direction:ltr;color:#333;text-align:left;mso-line-height-rule:exactly;mso-text-raise:2px}h1{margin:0;Margin:0;font-family:Roboto,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:34px;font-weight:400;font-style:normal;font-size:28px;text-decoration:none;text-transform:none;letter-spacing:0;direction:ltr;color:#333;text-align:left;mso-line-height-rule:exactly;mso-text-raise:2px}h2{margin:0;Margin:0;font-family:Lato,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:30px;font-weight:400;font-style:normal;font-size:24px;text-decoration:none;text-transform:none;letter-spacing:0;direction:ltr;color:#333;text-align:left;mso-line-height-rule:exactly;mso-text-raise:2px}h3{margin:0;Margin:0;font-family:Lato,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:26px;font-weight:400;font-style:normal;font-size:20px;text-decoration:none;text-transform:none;letter-spacing:0;direction:ltr;color:#333;text-align:left;mso-line-height-rule:exactly;mso-text-raise:2px}div.t23,div.t36{max-width:600px !important}
</style>
<![endif]-->
<!--[if mso]>
<xml>
<o:OfficeDocumentSettings>
<o:AllowPNG/>
<o:PixelsPerInch>96</o:PixelsPerInch>
</o:OfficeDocumentSettings>
</xml>
<![endif]-->
</head>
<body id=body class=t40 style="min-width:100%;Margin:0px;padding:0px;background-color:#F0F0F0;"><div class=t39 style="background-color:#F0F0F0;"><table role=presentation width=100% cellpadding=0 cellspacing=0 border=0 align=center><tr><td class=t38 style="font-size:0;line-height:0;mso-line-height-rule:exactly;background-color:#F0F0F0;" valign=top align=center>
<!--[if mso]>
<v:background xmlns:v="urn:schemas-microsoft-com:vml" fill="true" stroke="false">
<v:fill color=#F0F0F0/>
</v:background>
<![endif]-->
<table role=presentation width=100% cellpadding=0 cellspacing=0 border=0 align=center id=innerTable><tr><td>
<table class=t26 role=presentation cellpadding=0 cellspacing=0 align=center><tr>
<!--[if !mso]><!--><td class=t25 style="background-color:#FFFFFF;">
<!--<![endif]-->
<!--[if mso]><td class=t25 style="background-color:#FFFFFF;"><![endif]-->
<div class=t24 style="display:inline-table;width:100%;text-align:center;vertical-align:top;">
<!--[if mso]>
<table role=presentation cellpadding=0 cellspacing=0 align=center valign=top width=600><tr><td width=600 valign=top><![endif]-->
<div class=t23 style="display:inline-table;text-align:initial;vertical-align:inherit;width:100%;max-width:480px;">
<table role=presentation width=100% cellpadding=0 cellspacing=0 class=t22><tr>
<td class=t21><table role=presentation width=100% cellpadding=0 cellspacing=0><tr><td><div class=t1 style="mso-line-height-rule:exactly;mso-line-height-alt:125px;line-height:125px;font-size:1px;display:block;">&nbsp;</div></td></tr><tr><td>
<table class=t3 role=presentation cellpadding=0 cellspacing=0 align=center><tr>
<!--[if !mso]><!--><td class=t2 style="width:152px;">
<!--<![endif]-->
<!--[if mso]><td class=t2 style="width:152px;"><![endif]-->
<div style="font-size:0px;"><img class=t0 style="display:block;border:0;height:auto;width:100%;Margin:0;max-width:100%;" width=152 height=152 alt="" src="https://57648e28-030b-4148-9b22-9a9ee950327b.b-cdn.net/e/72cf2365-e68b-4774-928e-b6e7563f4233/db402102-74e9-4fa0-995c-fe763b110c1d.png"/></div></td>
</tr></table>
</td></tr><tr><td><div class=t5 style="mso-line-height-rule:exactly;mso-line-height-alt:55px;line-height:55px;font-size:1px;display:block;">&nbsp;</div></td></tr><tr><td>
<table class=t7 role=presentation cellpadding=0 cellspacing=0 align=center><tr>
<!--[if !mso]><!--><td class=t6 style="width:375px;">
<!--<![endif]-->
<!--[if mso]><td class=t6 style="width:375px;"><![endif]-->
<h1 class=t4 style="margin:0;Margin:0;font-family:Fira Sans,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:52px;font-weight:700;font-style:normal;font-size:48px;text-decoration:none;text-transform:none;direction:ltr;color:#000000;text-align:center;mso-line-height-rule:exactly;mso-text-raise:1px;">Kod autoryzacji</h1></td>
</tr></table>
</td></tr><tr><td><div class=t8 style="mso-line-height-rule:exactly;mso-line-height-alt:30px;line-height:30px;font-size:1px;display:block;">&nbsp;</div></td></tr><tr><td>
<table class=t11 role=presentation cellpadding=0 cellspacing=0 align=center><tr>
<!--[if !mso]><!--><td class=t10 style="width:350px;">
<!--<![endif]-->
<!--[if mso]><td class=t10 style="width:350px;"><![endif]-->
<p class=t9 style="margin:0;Margin:0;font-family:Fira Sans,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:30px;font-weight:500;font-style:normal;font-size:20px;text-decoration:none;text-transform:none;direction:ltr;color:#666666;text-align:center;mso-line-height-rule:exactly;mso-text-raise:3px;">Aby zalogować się na swoje konto wpisz poniższy kod na stronie.</p></td>
</tr></table>
</td></tr><tr><td><div class=t13 style="mso-line-height-rule:exactly;mso-line-height-alt:40px;line-height:40px;font-size:1px;display:block;">&nbsp;</div></td></tr><tr><td>
<table class=t15 role=presentation cellpadding=0 cellspacing=0 align=center><tr>
<!--[if !mso]><!--><td class=t14 style="background-color:#0055FF;overflow:hidden;width:308px;text-align:center;line-height:58px;mso-line-height-rule:exactly;mso-text-raise:11px;border-radius:14px 14px 14px 14px;">
<!--<![endif]-->
<!--[if mso]><td class=t14 style="background-color:#0055FF;overflow:hidden;width:308px;text-align:center;line-height:58px;mso-line-height-rule:exactly;mso-text-raise:11px;border-radius:14px 14px 14px 14px;"><![endif]-->
<code class=t12 style="display:block;margin:0;Margin:0;font-family:Fira Sans,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:58px;font-weight:600;font-style:normal;font-size:21px;text-decoration:none;direction:ltr;color:#FFFFFF;text-align:center;mso-line-height-rule:exactly;mso-text-raise:11px;" target=_blank>"#.to_owned() + code.as_str() + r#"</code></td>
</tr></table>
</td></tr><tr><td><div class=t16 style="mso-line-height-rule:exactly;mso-line-height-alt:60px;line-height:60px;font-size:1px;display:block;">&nbsp;</div></td></tr><tr><td>
<table class=t19 role=presentation cellpadding=0 cellspacing=0 align=center><tr>
<!--[if !mso]><!--><td class=t18 style="width:350px;">
<!--<![endif]-->
<!--[if mso]><td class=t18 style="width:350px;"><![endif]-->
<p class=t17 style="margin:0;Margin:0;font-family:Fira Sans,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:25px;font-weight:400;font-style:normal;font-size:16px;text-decoration:none;text-transform:none;direction:ltr;color:#BBBBBB;text-align:center;mso-line-height-rule:exactly;mso-text-raise:3px;">Jeżeli nie ty próbowałeś się zalogować możesz zignorować wiadomość albo zgłosić ją na https://ckziucodefest.pl/</p></td>
</tr></table>
</td></tr><tr><td><div class=t20 style="mso-line-height-rule:exactly;mso-line-height-alt:125px;line-height:125px;font-size:1px;display:block;">&nbsp;</div></td></tr></table></td>
</tr></table>
</div>
<!--[if mso]>
</td>
</tr></table>
<![endif]-->
</div></td>
</tr></table>
</td></tr><tr><td><div class=t37 style="display:inline-table;width:100%;text-align:center;vertical-align:top;">
<!--[if mso]>
<table role=presentation cellpadding=0 cellspacing=0 align=center valign=top width=600><tr><td width=600 valign=top><![endif]-->
<div class=t36 style="display:inline-table;text-align:initial;vertical-align:inherit;width:100%;max-width:480px;">
<table role=presentation width=100% cellpadding=0 cellspacing=0 class=t35><tr>
<td class=t34 style="padding:40px 0 40px 0;"><table role=presentation width=100% cellpadding=0 cellspacing=0><tr><td>
<table class=t29 role=presentation cellpadding=0 cellspacing=0 align=center><tr>
<!--[if !mso]><!--><td class=t28 style="width:350px;">
<!--<![endif]-->
<!--[if mso]><td class=t28 style="width:350px;"><![endif]-->
<p class=t27 style="margin:0;Margin:0;font-family:Fira Sans,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:19px;font-weight:400;font-style:normal;font-size:12px;text-decoration:none;text-transform:none;direction:ltr;color:#BBBBBB;text-align:center;mso-line-height-rule:exactly;mso-text-raise:2px;">Flash is a webtool that is a free open source JavaScript framework that can be accessed from a browser or mobile device in a Web browser.</p></td>
</tr></table>
</td></tr><tr><td><div class=t30 style="mso-line-height-rule:exactly;mso-line-height-alt:20px;line-height:20px;font-size:1px;display:block;">&nbsp;</div></td></tr><tr><td>
<table class=t33 role=presentation cellpadding=0 cellspacing=0 align=center><tr>
<!--[if !mso]><!--><td class=t32 style="width:350px;">
<!--<![endif]-->
<!--[if mso]><td class=t32 style="width:350px;"><![endif]-->
<p class=t31 style="margin:0;Margin:0;font-family:Fira Sans,BlinkMacSystemFont,Segoe UI,Helvetica Neue,Arial,sans-serif;line-height:19px;font-weight:400;font-style:normal;font-size:12px;text-decoration:none;text-transform:none;direction:ltr;color:#BBBBBB;text-align:center;mso-line-height-rule:exactly;mso-text-raise:2px;">Flash Inc. All rights reserved</p></td>
</tr></table>
</td></tr></table></td>
</tr></table>
</div>
<!--[if mso]>
</td>
</tr></table>
<![endif]-->
</div></td></tr></table></td></tr></table></div></body>
</html>"#)
    .unwrap();

  let mailer: SmtpTransport = SmtpTransport::relay("ssl0.ovh.net").expect("Failed to connect to SMTP")
    .credentials(Credentials::new("noreply@ckziucodefest.pl".into(), "FJWuSbmWKeRpePqX".into()))
    .build();

  let result = mailer.send(&email);
  println!("{:?}", result);
}