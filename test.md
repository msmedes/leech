I need to convert a `[u8; 20]` to `urlencoded` for an http request using `reqwest`, and I'm failing to convert properly.
`reqwest` allows you to pass a `[(&str, &str)]` to a query parameter encoder so 
`("info_hash", [64, 144, 195, 194, 163, 148, 164, 153, 116, 223, 187, 242, 206, 122, 208, 219, 60, 222, 221, 215])` should become
the `urlencoded` string `info_hash=%40%90%C3%C2%A3%94%A4%99t%DF%BB%F2%CEz%D0%DB%3C%DE%DD%D7`.

I've tried `String::from_utf8_lossy()` but that returns `%40%EF%BF%BD%EF%BF%BD%C2%A3%EF%BF%BD%EF%BF%BD%EF%BF%BDt%DF%BB%EF%BF%BD%EF%BF%BDz%EF%BF%BD%EF%BF%BD%3C%EF%BF%BD%EF%BF%BD%EF%BF%B` which is wrong and also way too long, so I'm sure I'm fundamentally misunderstanding the correct way to go about this.