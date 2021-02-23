# micro-cms

`micro-cms` is a simple headless CMS intended to be driven by a git repository.

This is currently used to power [my blog](https://blog.josbox.dev/) from [this repository](https://github.com/veryjos/blog-content).

**note:** `micro-cms` is currently a very experimental toy project and subject to many breaking API changes.

### Features
 - Content schema defined in [TOML](https://github.com/toml-lang/toml) for content validation and references between entities. 
 - Simple, query-only API supporting filtering and sorting of content.
 - Hot reloading.

### Next-Steps
 - GraphQL-based API using [juniper](https://github.com/graphql-rust/juniper) (blocked on missing dynamic schema support).
 
### How-To

Create a content folder with the following skeleton:

```
content/
|  Author/
|    schema
|    veryjos.ent
```

To create an entity type, create a folder and define the schema using TOML:

```toml
# Author/schema - defines the schema of the Author entity

[fields]
email = "str"
full_name = "str"
nickname = "str"
```

To define an Author entity, create another file in the `Author` folder using TOML:

```toml
# Author/veryjos.ent - defines an entity of the Author type

email = "jdelgado002@gmail.com"
full_name = "Joseph Delgado"
nickname = "veryjos"
```

Similarly, entities can be created using folders:

```
content/
|  Post/
|    schema
|    my_first_post/
|      ent
|      content.md
|      thumbnail.png
```

Post schema:

```toml
# Post/schema - defines an entity of the Post type

[fields]
title = "str"
author = "Author" # <- Entity reference
content = "str"
thumbnail = "bin" # The "bin" type cna be used to serve binary files, such as
                  # images. The webserver will automatically select the
                  # correct MIME type from the file in the repository.
```

Create a folder in your repository called `my_first_post`. Each file here will correspond to a field.

```
# Post/my_first_post/content.md <- defines the "content" field

post contents in *markdown* :)
```

Any fields that don't easily map to files in your repository can be supplied in TOML using a special file called `ent`:

```toml
# Post/my_first_post/ent

title = "My first post!"
author = "veryjos" # <- refer to other entities by ID
```

Finally, start the webserver:

```bash
./micro-cms \
    --content_path content \
    --bind_address 0.0.0.0 \
    --port 8080
 ```

For more arguments, run `./micro_cms --help`.

### API

For now, `micro-cms` uses a basic API that will eventually be migrated to a GraphQL API using [juniper](https://github.com/graphql-rust/juniper) once dynamic schemas are supported.

```
# Gets a single entity and specified fields.
GET /ent/<ty>/<ent_id>?fields=field_a,field_b,field_c...

# Gets a single entity and one field.
# This endpoint will automatically select the correct MIME type for the field.
GET /ent/<ty>/<ent_id>/<field_name>

# Request entity fields using a JSON POST body, similar to GraphQL.
POST /query

Example POST body:
{
  "Post": {
    "sort": { "by": "publish_date" },
    "fields": [ "title", "author" ]
  }
}

Example response:
{
  "Post": [
    {
      "title": "My first post!",
      "author": {
        "id": "veryjos",
        "email": "jdelgado002@gmail.com",
        "full_name": "Joseph Delgado",
        "nickname": "veryjos"
      }
    },
    {
      "title": "My second post!",
      "author": {
        "id": "veryjos",
        "email": "jdelgado002@gmail.com",
        "full_name": "Joseph Delgado",
        "nickname": "veryjos",
      }
    }
  ]
}
