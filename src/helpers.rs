use actix_web::web;
use actix_web::dev::*;

pub fn get_app_data<'a, T: 'static, B>(response: &'a ServiceResponse<B>) -> &'a T {
	&response.request().app_data::<web::Data<T>>().unwrap()
}