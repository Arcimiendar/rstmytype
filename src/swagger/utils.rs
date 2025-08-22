use serde_yaml_ng;
use utoipa::openapi::{ArrayBuilder, Type, schema::ObjectBuilder};

enum ObjectOrArrayBuilder {
    Object(ObjectBuilder),
    Array(ArrayBuilder),
}

fn parse_type(field: &serde_yaml_ng::Value) -> ObjectOrArrayBuilder {
    let mut object = ObjectBuilder::new();

    let description_opt = field
        .get("description")
        .and_then(|d_v| d_v.as_str())
        .or(Some(""));

    let field_type = field
        .get("type")
        .and_then(|t| t.as_str())
        .or(Some("string"));

    object = match field_type {
        Some("string" | "timestamp") => {
            object = object.schema_type(Type::String);

            let enum_v_opt = field
                .get("enum")
                .and_then(|e| e.as_sequence())
                .and_then(|e| {
                    Some(
                        e.iter()
                            .map(|v| v.as_str())
                            .flat_map(|v| v)
                            .collect::<Vec<&str>>(),
                    )
                });

            if let Some(enum_v) = enum_v_opt {
                object = object.enum_values(Some(enum_v))
            }

            object
        }
        Some("number" | "integer") => object.schema_type(Type::Number),
        Some("boolean" | "bool") => object.schema_type(Type::Boolean),
        Some("object") => {
            let inner_fields_opt = field.get("fields").and_then(|f| f.as_sequence());

            if let Some(inner_fields) = inner_fields_opt {
                for inner_field in inner_fields {
                    let name_opt = inner_field.get("field").and_then(|f| f.as_str());
                    if name_opt.is_none() {
                        continue;
                    }
                    let Some(name) = name_opt else {
                        continue;
                    };
                    let optional_flag_opt = inner_field
                        .get("optional")
                        .and_then(|o| o.as_bool())
                        .or(Some(false));

                    let inner_type = parse_type(inner_field);
                    match inner_type {
                        ObjectOrArrayBuilder::Object(o) => {
                            object = object.property(name, o);
                        }
                        ObjectOrArrayBuilder::Array(a) => {
                            object = object.property(name, a);
                        }
                    }

                    if let Some(optional_flag) = optional_flag_opt
                        && optional_flag
                    {
                        object = object.required(name);
                    }
                }
            }

            object
        }
        Some("array") => {
            let arr = object.to_array_builder();
            let items_opt = field.get("items");

            if let Some(items) = items_opt {
                let arr_type = parse_type(items);

                let parsed_arr = match arr_type {
                    ObjectOrArrayBuilder::Array(a) => a.to_array_builder(),
                    ObjectOrArrayBuilder::Object(o) => o.to_array_builder(),
                };
                return ObjectOrArrayBuilder::Array(parsed_arr);
            }

            return ObjectOrArrayBuilder::Array(arr);
        }
        _ => object,
    };

    ObjectOrArrayBuilder::Object(object.description(description_opt))
}

pub fn append_field(field: &serde_yaml_ng::Value, object: ObjectBuilder) -> ObjectBuilder {
    let name_opt = field.get("field").and_then(|n| n.as_str());

    let Some(name) = name_opt else {
        return object;
    };

    let field_type = parse_type(&field);

    match field_type {
        ObjectOrArrayBuilder::Array(a) => object.property(name, a),
        ObjectOrArrayBuilder::Object(o) => object.property(name, o),
    }
    .required(name)
}
