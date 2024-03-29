use actix_web::{
    HttpResponse, Responder, Result, web, Scope
};


use crate::presentation::handler::control::model::datakey::dto::{DataKeyDTO, ExportKey};
use crate::util::error::Error;
use validator::Validate;
use crate::application::datakey::KeyService;
use super::model::user::dto::UserIdentity;


async fn create_data_key(_user: UserIdentity, key_service: web::Data<dyn KeyService>, datakey: web::Json<DataKeyDTO>,) -> Result<impl Responder, Error> {
    datakey.validate()?;
    Ok(HttpResponse::Created().json(DataKeyDTO::try_from(key_service.into_inner().create(datakey.0).await?)?))
}

async fn list_data_key(_user: UserIdentity, key_service: web::Data<dyn KeyService>) -> Result<impl Responder, Error> {
    let keys = key_service.into_inner().get_all().await?;
    let mut results = vec![];
    for k in keys {
        results.push(DataKeyDTO::try_from(k)?)
    }
    Ok(HttpResponse::Ok().json(results))
}

async fn show_data_key(_user: UserIdentity, key_service: web::Data<dyn KeyService>, id: web::Path<String>) -> Result<impl Responder, Error> {
    let key = key_service.into_inner().get_one(id.parse::<i32>()?).await?;
    Ok(HttpResponse::Ok().json(DataKeyDTO::try_from(key)?))
}

async fn delete_data_key(_user: UserIdentity, key_service: web::Data<dyn KeyService>, id: web::Path<String>) -> Result<impl Responder, Error> {
    key_service.into_inner().delete_one(id.parse::<i32>()?).await?;
    Ok(HttpResponse::Ok())
}

async fn export_data_key(_user: UserIdentity, key_service: web::Data<dyn KeyService>, id: web::Path<String>) -> Result<impl Responder, Error> {
    Ok(HttpResponse::Ok().json(ExportKey::try_from(key_service.export_one(id.parse::<i32>()?).await?)?))
}

async fn enable_data_key(_user: UserIdentity, key_service: web::Data<dyn KeyService>, id: web::Path<String>) -> Result<impl Responder, Error> {
    key_service.enable(id.parse::<i32>()?).await?;
    Ok(HttpResponse::Ok())
}

async fn disable_data_key(_user: UserIdentity, key_service: web::Data<dyn KeyService>, id: web::Path<String>) -> Result<impl Responder, Error> {
    key_service.disable(id.parse::<i32>()?).await?;
    Ok(HttpResponse::Ok())
}

async fn import_data_key(_user: UserIdentity) -> Result<impl Responder, Error> {
    Ok(HttpResponse::Ok())
}


pub fn get_scope() -> Scope {
    web::scope("/keys")
        .service(
            web::resource("/")
                .route(web::get().to(list_data_key))
                .route(web::post().to(create_data_key)))
        .service( web::resource("/{id}")
            .route(web::get().to(show_data_key))
            .route(web::delete().to(delete_data_key)))
        .service( web::resource("/import").route(web::post().to(import_data_key)))
        .service( web::resource("/{id}/export").route(web::post().to(export_data_key)))
        .service( web::resource("/{id}/enable").route(web::post().to(enable_data_key)))
        .service( web::resource("/{id}/disable").route(web::post().to(disable_data_key)))
}
