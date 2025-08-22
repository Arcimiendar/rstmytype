mod swagger;

pub use crate::swagger::{
    build_open_api,
    types::{ApiEndpoint, ApiEndpointMethod, ApiProject},
};

#[cfg(test)]
mod tests {
    use super::*;

    struct ApiProjectTest {
        api_endpoints: Vec<ApiEndpointTest>,
    }
    struct ApiEndpointTest {
        api_endpoint_method: ApiEndpointMethod,
        url_path: String,
        api_declaration: Option<String>,
    }

    impl ApiProjectTest {
        fn new(api_endpoints: Vec<ApiEndpointTest>) -> Self {
            Self { api_endpoints }
        }
    }

    impl ApiProject for ApiProjectTest {
        fn get_title(&self) -> &str {
            "Test"
        }

        fn get_endpoints_iter<'a>(&'a self) -> impl Iterator<Item = &'a impl ApiEndpoint> {
            self.api_endpoints.iter()
        }
    }

    impl ApiEndpointTest {
        pub fn new(
            method: ApiEndpointMethod,
            url_path: String,
            api_declaration: Option<String>,
        ) -> Self {
            Self {
                api_endpoint_method: method,
                url_path,
                api_declaration,
            }
        }
    }

    impl ApiEndpoint for ApiEndpointTest {
        fn get_url_path(&self) -> &str {
            &self.url_path
        }

        fn get_endpoint_method(&self) -> &ApiEndpointMethod {
            &self.api_endpoint_method
        }

        fn get_yml_declaration_str(&self) -> Option<&str> {
            if let Some(decl) = &self.api_declaration {
                return Some(&decl);
            }

            return None;
        }
    }

    #[test]
    fn smocking_test() {
        let api_project = ApiProjectTest::new(vec![]);
        build_open_api(&api_project);
    }

    #[test]
    fn broken_declaration_is_ok() {
        let api_project = ApiProjectTest::new(vec![ApiEndpointTest::new(
            ApiEndpointMethod::Get,
            "/1/2/3".to_owned(),
            Some("it's not declaration".to_owned()),
        )]);
        build_open_api(&api_project);
    }

    #[test]
    fn get_declaration_is_ok() {
        let api_project = ApiProjectTest::new(vec![ApiEndpointTest::new(
            ApiEndpointMethod::Get,
            "/1/2/3".to_owned(),
            Some(
                r#"
                    declaration:
                      description: test get params
                      allowlist:
                        query:
                          - field: hello
                            type: integer
                      response:
                        fields:
                          - field: ?column?
                            description: test unnamed column
                            type: integer
                "#
                .to_owned(),
            ),
        )]);

        build_open_api(&api_project);
    }

    #[test]
    fn post_declaration_is_ok() {
        let api_project = ApiProjectTest::new(vec![ApiEndpointTest::new(
            ApiEndpointMethod::Get,
            "/1/2/3".to_owned(),
            Some(
                r#"
                  declaration:
                    description: test post params
                    allowlist:
                      body:
                        - field: one
                          type: integer
                        - field: two
                          type: object
                        - field: three
                          type: array
                          items: 
                            type: integer
                    response:
                      fields:
                        - field: one
                          type: integer
                        - field: two
                          type: object
                        - field: three
                          type: array
                          items: 
                            type: integer
                "#
                .to_owned(),
            ),
        )]);

        build_open_api(&api_project);
    }
}
