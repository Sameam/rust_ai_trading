use actix_web::{web, HttpResponse, Responder};
use std::{sync::Arc};
use serde::{Serialize, Deserialize};

use crate::{ app::{controller::agent_controllers::AgentController}};

#[derive(Deserialize, Serialize)]
pub struct AgentHedgeFundRequest {
  tickers: Vec<String>,
  start_date: Option<String>,
  end_date: Option<String>,
  initial_cash: Option<f64>,
  margin_requirement: Option<f64>,
  show_reasoning: Option<bool>,
  selected_analysts: Option<Vec<String>>,
  model_name: Option<String>,
  model_provider: Option<String>,
}


pub struct Routes;

impl Routes {

  #[allow(unused)]
  pub fn new() -> Self {
    Routes {}
  }

  pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(Self::health)));
    cfg.service(web::resource("/agent/analysts").route(web::get().to(Self::get_analysts)));
    cfg.service(web::resource("/agent/models").route(web::get().to(Self::get_models)));
    cfg.service(web::resource("/agent/investment").route(web::post().to(Self::hedge_fund)));
  }

  async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
      "status": "ok",
      "Info": "Welcome to Rust AI_HedgeFund.", 
      "code": 200,
    }))
  }

  async fn get_analysts(controller: web::Data<Arc<AgentController>>) -> impl Responder {
    match controller.get_available_analysts().await {
      Ok(analysts) => HttpResponse::Ok().json(analysts),
      Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
  }

  async fn get_models(controller: web::Data<Arc<AgentController>>) -> impl Responder {
    match controller.get_available_model().await {
      Ok(model) => HttpResponse::Ok().json(model),
      Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
  }

  async fn hedge_fund(controller: web::Data<Arc<AgentController>>, request: web::Json<AgentHedgeFundRequest>) -> impl Responder {
    // let tickers = request.tickers.clone
    let tickers = request.tickers.clone();
    let start_date = request.start_date.as_deref();
    let end_date = request.end_date.as_deref(); 

    let selected_analysts = request.selected_analysts.clone();
    let model_name = request.model_name.clone();
    let model_provider = request.model_provider.clone();

    let result = controller.hedge_fund(tickers, start_date, end_date, request.initial_cash, request.margin_requirement, request.show_reasoning, selected_analysts, model_name, model_provider).await;

    match result {
      Ok(data) => HttpResponse::Ok().json(data),
      Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
          "error": e.to_string(),
      }))
    }


  }


}