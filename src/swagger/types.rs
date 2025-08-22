#[derive(PartialEq)]
pub enum ApiEndpointMethod {
    Get,
    Post,
}

pub trait ApiEndpoint {
    fn get_url_path(&self) -> &str;
    fn get_endpoint_method(&self) -> &ApiEndpointMethod;
    fn get_endpoint_tag(&self) -> &str {
        return "api";
    }

    fn get_yml_declaration_str(&self) -> Option<&str>;
}

pub trait ApiProject {
    fn get_title(&self) -> &str;

    fn get_version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn get_endpoints_iter<'a>(&'a self) -> impl Iterator<Item = &'a impl ApiEndpoint>;
}
