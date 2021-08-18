use crate::http;
use crate::utils;
use crate::types::*;
use crate::error::*;

pub async fn subscribe(
    ctx: &Context,
    form: &UrlSearchParams,
) -> Result<Response> {
    type Param = Option<String>;
    let token: Param = form.get("token");
    let proto: Param = form.get("proto");
    if token.is_none() || proto.is_none() {
        return Ok(http::not_found());
    }
    let valid_token = utils::md5sum(&utils::month().to_string());
    if token.unwrap() != valid_token {
        return Ok(http::not_found());
    }
    let (kv, proto) = match proto.unwrap().as_str() {
        "v2" => (&ctx.kv_v2, "v2ray"),
        "ss" => (&ctx.kv_ss, "shadowsocks"),
        _ => return Ok(http::not_found()),
    };
    let res: JsValue =
        kv.list(JsValue::NULL, JsValue::NULL, JsValue::NULL).await?;
    let res: ListResult = res.into_serde()?;

    /*let mut text = Vec::<String>::new();
    for key in res.keys {
        let link: JsValue = kv.get(key.name.into(), JsValue::NULL).await?;
        let link: String = link.into_serde()?;
        text.push(link)
    }*/
    let text: Vec<String> =
        futures::future::try_join_all(res.keys.into_iter().map(|key| async {
            let link: JsValue =
                match kv.get(key.name.into(), JsValue::NULL).await {
                    Ok(x) => x,
                    Err(e) => return Err(e),
                };
            let link: String = match link.into_serde() {
                Ok(x) => x,
                Err(e) => return Err(e.to_string().into()),
            };
            Ok(link)
        }))
        .await?;
    Ok(http::new_response(&text.join("\n")))
}
