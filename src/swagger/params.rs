use serde_yaml_ng;
use utoipa::openapi::{
    Content, ObjectBuilder, Required, Type,
    path::{Parameter, ParameterBuilder, ParameterIn},
    request_body::{RequestBody, RequestBodyBuilder},
    schema::Schema,
};

use crate::swagger::utils::append_field;

pub fn get_query_params(declaration: &serde_yaml_ng::Mapping) -> Option<Vec<Parameter>> {
    let query_arr = declaration
        .get("allowlist")
        .and_then(|a| a.get("query"))
        .and_then(|q| q.as_sequence())?;

    Some(
        query_arr
            .iter()
            .map(|qp| {
                let name = qp.get("field").and_then(|f| f.as_str())?;

                let description = qp.get("description").and_then(|d| d.as_str());

                // change it if query params will be parsed from server side, not DB side
                let qp_type = Type::String;

                let param = ParameterBuilder::new()
                    .name(name.to_string())
                    .parameter_in(ParameterIn::Query)
                    .description(description)
                    .required(Required::True)
                    .schema(Some(Schema::Object(
                        ObjectBuilder::new().schema_type(qp_type).build(),
                    )))
                    .build();
                Some(param)
            })
            .flat_map(|v| v)
            .collect(),
    )
}

pub fn get_request_body(declaration: &serde_yaml_ng::Mapping) -> Option<(RequestBody, Schema)> {
    let field_arr = declaration
        .get("allowlist")
        .and_then(|a| a.get("body"))
        .and_then(|b| b.as_sequence())?;

    let mut object = ObjectBuilder::new();

    for field in field_arr {
        object = append_field(&field, object);
    }

    let schema = Schema::Object(object.build());

    let request_body = RequestBodyBuilder::new()
        .content("application/json", Content::new(Some(schema.clone())))
        .required(Some(Required::True))
        .build();

    Some((request_body, schema))
}
