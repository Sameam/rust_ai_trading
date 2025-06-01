use actix_web::{web, App};
use std::sync::Arc;

use crate::app::config::Config;
use crate::app::routes::routes::Routes;

use super::controller::agent_controllers::AgentController;
use super::services::agent_service::AgentService;
use super::services::service::HedgeFundServices;

#[derive(Clone)]
pub struct AppState {
  pub agent_controller: Arc<AgentController>
}

impl AppState {

  #[allow(unused)]
  pub fn new(app_config: &Config) -> Self {
    let agent_service : AgentService = AgentService::new(app_config.clone());
    let hedge_fund_service: Arc<HedgeFundServices> = Arc::new(HedgeFundServices::new(agent_service));
    let agent_controller : Arc<AgentController> = Arc::new(AgentController::new(hedge_fund_service.clone()));
    AppState { agent_controller }
  }
}

#[allow(unused)]
pub struct CreateApp {
  app_state: AppState,
  app_settings: Config,
}

impl CreateApp {
  pub fn new(app_settings: Config) -> Self {
    let app_state: AppState = AppState::new(&app_settings);
    CreateApp { app_state, app_settings  }
  }

  pub fn build_app(&self,) -> App<impl actix_web::dev::ServiceFactory<actix_web::dev::ServiceRequest,Config = (),Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,Error = actix_web::Error,InitError = (),>,> {
    App::new()
    .app_data(web::Data::new(self.app_state.agent_controller.clone()))
    .configure(Routes::configure)
  }
}
