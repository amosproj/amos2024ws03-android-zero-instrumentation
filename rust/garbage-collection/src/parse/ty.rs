// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use clang::Entity;

use super::field::Field;

#[derive(Debug)]
pub struct Type {
    fields: HashMap<String, Field>,
    size: Option<usize>,
}

impl Type {
    pub fn parse(entity: Entity) -> Self {
        let mut fields = HashMap::new();

        let ty = entity.get_type().unwrap();
        let size = ty.get_sizeof().ok();

        ty.visit_fields(|field| {
            if let Some(name) = field.get_name() {
                let field = Field::parse(field);
                fields.insert(name, field);
            } else {
                eprintln!("missing name: {:?}", field);
            }
            true
        });

        Self { fields, size }
    }

    pub fn try_get_field(&self, name: &str) -> Option<&Field> {
        self.fields.get(name)
    }

    pub fn try_get_size(&self) -> Option<usize> {
        self.size
    }

    pub fn get_field(&self, name: &str) -> &Field {
        self.try_get_field(name).expect("type has no field")
    }

    pub fn get_size(&self) -> usize {
        self.try_get_size().expect("type has no size")
    }
}
