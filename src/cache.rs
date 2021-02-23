use std::collections::HashMap;

use crate::entity::{Entity, FieldType, FieldData};
use crate::schema::EntityDeclaration;

pub struct Cache {
    entities: HashMap<String, TypeGroup>,
}

pub struct TypeGroup {
    pub declaration: EntityDeclaration,
    pub entities: HashMap<String, Entity>,
}

impl TypeGroup {
    fn new(declaration: EntityDeclaration) -> TypeGroup {
        TypeGroup {
            declaration,
            entities: HashMap::new(),
        }
    }

    pub fn get_entity(&self, name: &str) -> &Entity {
        self.entities.get(name).unwrap()
    }

    fn add_entity(&mut self, name: &str, ent: Entity) {
        self.entities.insert(name.to_owned(), ent);
    }
}

pub fn validate_entity<'a>(cache: &'a Cache, decl: &'a EntityDeclaration, entity: &'a Entity) {
    for (key, val) in entity.fields.iter() {
        // Validate the field exists
        let field = match decl.fields.get(key) {
            Some(field) => field,
            None => {
                println!(r#"No such field "{key}" on entity"#);
                continue;
            }
        };

        // Validate the type of the field
        match (&field.ty, val) {
            (FieldType::Str, FieldData::Str(_)) |
            (FieldType::Bin, FieldData::Bin(_)) |
            (FieldType::Num, FieldData::Num(_)) => {},

            (FieldType::Ref(ty), FieldData::Str(ent_name)) => {
                // Validate the reference entity type exists.
                cache.entities.get(ty)
                    .expect(&format!(r#"No such entity type "{:?}""#, ty));

                // Validate the reference's entity exists.
                cache.entities.get(ent_name)
                    .expect(&format!(r#"No such entity "{}" of type "{:?}""#, ent_name, ty));
            },

            (_, _) => panic!(r#"Field "{}" declared as "{:?}", but "{:?}" was provided as value"#, key, field.ty, val)
        };
    }
}

pub fn validate_type_group<'a>(cache: &'a Cache, type_group: &'a TypeGroup) {
    for (_, entity) in type_group.entities.iter() {
        validate_entity(cache, &type_group.declaration, &entity);
    }
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            entities: HashMap::new(),
        }
    }

    pub fn validated(self) -> Self {
        // Validate each type group
        for (_, group) in self.entities.iter() {
            validate_type_group(&self, group);
        }

        self
    }

    pub fn add_type(&mut self, name: &str, decl: EntityDeclaration) {
        self.entities.insert(name.to_owned(), TypeGroup::new(decl));
    }

    pub fn add_entity(&mut self, ty_name: &str, name: &str, ent: Entity) {
        let group = self.entities.get_mut(ty_name).unwrap();

        group.add_entity(name, ent);
    }

    pub fn get_group(&self, type_name: &str) -> &TypeGroup {
        self.entities.get(type_name).unwrap()
    }
}
