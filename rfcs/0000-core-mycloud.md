- Feature Name: my-cloud
- Start Date: 2023-03-28

### Summary
[summary]: #summary

Create a self-hosted cloud using rust.

### Motivation
[motivation]: #motivation

For practicing Rust and for personal use.

### Feature

#### 1. top priority
1. Upload/Download/Delete file to cloud

#### 2. todo
2. Preview file
3. Search file by title
4. Search file by content
5. Notify the client of file changes and synchronize files


### Archtecture
[archtecture]: #archtecture

![archtecture](imgs/mycloud-arch-0001.png)

#### 1. web service
##### 1. Upload
```
method: POST 
url: /api/store/{user_id}/{path}?is_dir=text
body: octream
response: 200,201, 409, 401
```

##### 2. Modify file or dir name
```
method: PUT 
url: /api/store/{user_id}/{resource_id}
body: formdata
    filename: text
response:
    200
    201
    401
    409
```


##### 2. Download
`GET /api/store/{user_id}/{resource_id}`

##### 3. Delete
`DELETE /api/store/{user_id}/{resource_id}`

##### 4. Get File List
```json
method: GET 
url: /api/store/{user_id}/{resource_id}
response: json
[
    {
        filename: string
        is_dir: boolean
        rid: string
        size: number
        type: string
    }
]
```

#### 2. core

File will be stored in the two parts: meta data and file content. 

##### 1. Meta data
Meta data is stored in file history table. Every time when a file is modified, a new record will be inserted into the file history table. The file history table will record the file version, slices and slices hash. Slices is ordered array and slices hash is the hash of every slice.

##### 2. File content
Every file content will be cut into slices and max size of every slice is 4MB and slice_id is uuid. Hash of slice can't be used as slice_id because the hash of slice is not unique. File content will be stored in the file system or s3 or azure blob or etc. Slices store sturcture is `slice_id[0]/slice_id[1]/slice_id`. This is to avoid some directories is too big to load slowly, but it remains to be verified.


#### 3. db schema
postgres sql
##### 1. user 
```sql

create table user table (
    id bigint not null primary key,
    username varchar(255) not null,
    password varchar(255) not null,
    email varchar(255) not null,
    create_time datetime default current_timestamp,
    update_time datetime default current_timestamp on update current_timestamp
);
```

##### 2. file
```sql
create table file (
    id bigint not null primary key,
    uid bigint not null,
    filename varchar(255) not null,
    parent_dir_id bigint not null,
    is_dir boolean not null,
    size bigint not null,
    create_time datetime default current_timestamp,
    -- update_time update on update
    update_time datetime default current_timestamp on update current_timestamp
);
```

##### 3. file history 
```sql
create table file_history (
    id bigint not null primary key,
    fid bigint not null,
    file_version int not null,
    -- slice is array
    slices text[] not null,
    slices_hash text[] not null,
    create_time datetime default current_timestamp,
    update_time datetime default current_timestamp on update current_timestamp
);
```


# Drawbacks
[drawbacks]: #drawbacks

Why should we *not* do this?

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

- Why is this design the best in the space of possible designs?
- What other designs have been considered and what is the rationale for not choosing them?
- What is the impact of not doing this?
- If this is a language proposal, could this be done in a library or macro instead? Does the proposed change make Rust code easier or harder to read, understand, and maintain?

# Prior art
[prior-art]: #prior-art

Discuss prior art, both the good and the bad, in relation to this proposal.
A few examples of what this can include are:

- For language, library, cargo, tools, and compiler proposals: Does this feature exist in other programming languages and what experience have their community had?
- For community proposals: Is this done by some other community and what were their experiences with it?
- For other teams: What lessons can we learn from what other communities have done here?
- Papers: Are there any published papers or great posts that discuss this? If you have some relevant papers to refer to, this can serve as a more detailed theoretical background.

This section is intended to encourage you as an author to think about the lessons from other languages, provide readers of your RFC with a fuller picture.
If there is no prior art, that is fine - your ideas are interesting to us whether they are brand new or if it is an adaptation from other languages.

Note that while precedent set by other languages is some motivation, it does not on its own motivate an RFC.
Please also take into consideration that rust sometimes intentionally diverges from common language features.

# Unresolved questions
[unresolved-questions]: #unresolved-questions

- What parts of the design do you expect to resolve through the RFC process before this gets merged?
- What parts of the design do you expect to resolve through the implementation of this feature before stabilization?
- What related issues do you consider out of scope for this RFC that could be addressed in the future independently of the solution that comes out of this RFC?

# Future possibilities
[future-possibilities]: #future-possibilities

Think about what the natural extension and evolution of your proposal would
be and how it would affect the language and project as a whole in a holistic
way. Try to use this section as a tool to more fully consider all possible
interactions with the project and language in your proposal.
Also consider how this all fits into the roadmap for the project
and of the relevant sub-team.

This is also a good place to "dump ideas", if they are out of scope for the
RFC you are writing but otherwise related.

If you have tried and cannot think of any future possibilities,
you may simply state that you cannot think of anything.

Note that having something written down in the future-possibilities section
is not a reason to accept the current or a future RFC; such notes should be
in the section on motivation or rationale in this or subsequent RFCs.
The section merely provides additional information.
Symbols
Symbol outline not available for this file
To inspect a symbol, try clicking on the symbol directly in the code view.
Code navigation supports a limited number of languages. See which languages are supported.
