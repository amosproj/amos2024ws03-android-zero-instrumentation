// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use clang::{Entity, EntityKind, EntityVisitResult};

use super::ty::Type;


#[derive(Debug)]
pub struct Namespace {
    namespaces: HashMap<String, Namespace>,
    types: HashMap<String, Type>,
}

impl Namespace {
    fn new() -> Self {
        Self {
            namespaces: HashMap::new(),
            types: HashMap::new(),
        }
    }
    
    fn parse_namespace(&mut self, entity: Entity) {
        assert_eq!(entity.get_kind(), EntityKind::Namespace);
        if let Some(name) = entity.get_name() {
            let namespace = Namespace::parse(entity);
            self.namespaces.insert(name, namespace);
        } else {
            eprintln!("missing name: {:?}", entity);
        }
    }
    
    fn parse_struct(&mut self, entity: Entity) {
        assert!(matches!(entity.get_kind(), EntityKind::StructDecl | EntityKind::ClassDecl));

        if let Some(name) = entity.get_name() {
            let decl = entity
                .get_type()
                .expect("structs always have a type")
                .get_declaration()
                .expect("structs always have a declaration");

            self.types.insert(name, Type::parse(decl));
        } else {
            eprintln!("missing name: {:?}", entity);
        }
    }

    pub fn parse(entity: Entity) -> Self {
        let mut this = Self::new();
        
        entity.visit_children(|child, _| {
            match child.get_kind() {
                EntityKind::Namespace => this.parse_namespace(child),
                EntityKind::StructDecl | EntityKind::ClassDecl => this.parse_struct(child),
                _ => {}
            }
            
            EntityVisitResult::Continue
        });
        
        this
    }
    
    pub fn try_get_namespace(&self, name: &str) -> Option<&Namespace> {
        self.namespaces.get(name)
    }

    pub fn try_get_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }

    pub fn get_namespace(&self, name: &str) -> &Namespace {
        self.try_get_namespace(name).expect("namespace not found")
    }
    
    pub fn get_type(&self, name: &str) -> &Type {
        self.try_get_type(name).expect("type not found")
    }
}