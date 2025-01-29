// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use clang::Entity;

use super::ty::Type;

#[derive(Debug)]
pub struct Field {
    offset: Option<usize>,
    ty: Option<Type>,
}

impl Field {
    pub fn parse(entity: Entity) -> Self {
        let offset = entity.get_offset_of_field().ok();
        let declaration = entity
            .get_type()
            .expect("field always has a type")
            .get_declaration();
        let ty = declaration.map(Type::parse);

        Self { offset, ty }
    }

    pub fn try_get_offset(&self) -> Option<usize> {
        self.offset
    }

    pub fn try_get_type(&self) -> Option<&Type> {
        self.ty.as_ref()
    }

    pub fn get_offset(&self) -> usize {
        self.try_get_offset().expect("field has no offset")
    }

    pub fn get_type(&self) -> &Type {
        self.try_get_type().expect("field has no type")
    }
}
