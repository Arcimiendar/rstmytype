use std::collections::HashMap;

use convert_case::{self, Case, Casing};
use log::warn;
use serde_yaml_ng;
use utoipa::openapi::{
    ComponentsBuilder, HttpMethod, InfoBuilder, OpenApi, OpenApiBuilder, PathsBuilder, Schema,
    path::{Operation, OperationBuilder, PathItemBuilder},
};

use crate::swagger::params::{get_query_params, get_request_body};
use crate::swagger::response::get_response;
use crate::swagger::types::{ApiProject, ApiEndpoint, ApiEndpointMethod};

pub mod types;
mod params;
mod response;
mod utils;

fn get_operation_and_schema(
    endpoint: &impl ApiEndpoint,
) -> (Operation, HashMap<String, Schema>) {
    let mut operation = OperationBuilder::new()
        .operation_id(Some(endpoint.get_url_path()))
        .tag(endpoint.get_endpoint_tag());
    let mut schema_openapi_map = HashMap::new();

    let schema_str_opt = endpoint.get_yml_declaration_str();
    if schema_str_opt.is_none() {
        return (operation.build(), schema_openapi_map);
    }

    let schema_str = schema_str_opt.unwrap();
    // warn!("{}", schema_str);

    let schema_res = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&schema_str);

    if schema_res.is_err() {
        warn!(
            "Found declaration in {}, but it's maleformatted",
            endpoint.get_url_path()
        );
        return (operation.build(), schema_openapi_map);
    }
    let schema = schema_res.unwrap();

    let declaration_opt = schema
        .as_mapping()
        .and_then(|s| s.get("declaration"))
        .and_then(|d| d.as_mapping());

    if declaration_opt.is_none() {
        warn!(
            "Found declaration in {}, but it's maleformatted",
            endpoint.get_url_path()
        );
        return (operation.build(), schema_openapi_map);
    }
    let declaration = declaration_opt.unwrap();

    if let Some(description_val) = declaration.get("description") {
        if !description_val.is_string() {
            warn!(
                "Found declaration in {}, declaration.description must be a string",
                endpoint.get_url_path()
            );
        }

        let description = description_val.as_str().unwrap();
        operation = operation.description(Some(description.to_string()));
    }

    if let Some((response, schema)) = get_response(&declaration) {
        operation = operation.response("200", response);

        schema_openapi_map.insert(
            format!("Response{}", endpoint.get_url_path().replace("/", "_")).to_case(Case::UpperCamel),
            schema,
        );
    }

    if *endpoint.get_endpoint_method() == ApiEndpointMethod::Get {
        if let Some(params) = get_query_params(&declaration) {
            for param in params {
                operation = operation.parameter(param);
            }
        }
    } else if *endpoint.get_endpoint_method() == ApiEndpointMethod::Post {
        if let Some((request_body, schema)) = get_request_body(&declaration) {
            operation = operation.request_body(Some(request_body));

            schema_openapi_map.insert(
                format!("Post{}", endpoint.get_url_path().replace("/", "_")).to_case(Case::UpperCamel),
                schema,
            );
        }
    }

    (operation.build(), schema_openapi_map)
}

fn build_schema_for_endpoint(
    mut path_builder: PathsBuilder,
    mut components_builder: ComponentsBuilder,
    endpoint: &impl ApiEndpoint,
) -> (PathsBuilder, ComponentsBuilder) {
    let path_item_builder = PathItemBuilder::new();

    let (operation, schema_opt) = get_operation_and_schema(endpoint);

    for (key, value) in schema_opt {
        components_builder = components_builder.schema(key, value);
    }

    path_builder = path_builder.path(
        endpoint.get_url_path(),
        path_item_builder
            .operation(
                match endpoint.get_endpoint_method() {
                    ApiEndpointMethod::Get => HttpMethod::Get,
                    ApiEndpointMethod::Post => HttpMethod::Post,
                },
                operation,
            )
            .build(),
    );

    (path_builder, components_builder)
}

pub fn build_open_api(api_project: &impl ApiProject) -> OpenApi {
    let builder = OpenApiBuilder::new().info(
        InfoBuilder::new()
            .title(api_project.get_title())
            .version(api_project.get_version())
            .build(),
    );

    let mut components_builder = ComponentsBuilder::new();
    let mut paths_builder = PathsBuilder::new();

    for endpoint in api_project.get_endpoints_iter() {
        (paths_builder, components_builder) =
                build_schema_for_endpoint(paths_builder, components_builder, endpoint);
    }

    builder
        .paths(paths_builder.build())
        .components(Some(components_builder.build()))
        .build()
}
