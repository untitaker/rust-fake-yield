## fake-yield

Python's `yield` implemented for Rust. 

```rust
fn to_html_stream<'a>(xs: impl IntoIterator<Item = &'a str>) -> impl Iterator<Item = &'a str> {
    fake_yield! {{
        _yield!("<ul>\n");
        for x in xs {
            _yield!("<li>");
            _yield!(x);  // Don't try this at home
            _yield!("</li>\n");
        }
        _yield!("</ul>");
    }}
}
```

This is very much work-in-progress:

* Other kinds of loops don't work.
* If you use `break` or `continue`, weird things will happen.

Check `tests/` for more examples.

Probably affects performance.
