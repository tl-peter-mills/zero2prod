use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;
use actix_web::http::header::ContentType;
use std::fmt::Write;

pub async fn new_newsletter_form(
    flash_messages: IncomingFlashMessages
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Change Password</title>
</head>
<body>
    {}
    <form action="/admin/newsletters" method="post">
        <label>Title<br>
            <input
                placeholder="Enter the newsletter's title"
                name="title"
            >
        </label>
        <br>
        <label>HTML content<br>
            <textarea
                placeholder="Enter the html content of the newsletter"
                name="html_content"
            ></textarea>
        </label>
        <br>
        <label>Text content<br>
            <textarea
                placeholder="Enter the text content of the newsletter"
                name="text_content"
            ></textarea>
        </label>
        <br>
        <button type="submit">Send newsletter</button>
    </form>
    <p><a href="/admin/dashboard">&lt;- Back</a></p>
</body>
</html>"#,
            msg_html
        )))

}