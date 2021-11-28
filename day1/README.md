# Day 1

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus dignissim malesuada nisl, ac molestie sem elementum ac. Proin at mauris dapibus, tempor elit sed, hendrerit lectus. Vivamus eu massa dapibus, commodo orci nec, sagittis augue. Praesent dui lacus, consectetur sit amet ipsum et, aliquet consectetur turpis. Pellentesque iaculis posuere magna id placerat. Aliquam fringilla bibendum massa vitae venenatis. Sed vitae leo tempus, auctor elit in, eleifend lectus. Suspendisse potenti. Suspendisse vulputate felis mi, sed commodo sapien commodo in. Sed convallis leo in rutrum tincidunt. Sed tempor euismod massa ut congue. Proin sed bibendum velit. Morbi sollicitudin sagittis metus id vestibulum.

```rust
pub fn new() -> Component {
	// Create a new token called "HelloToken," with a fixed supply of 1000, and put that supply into a bucket
    	let my_bucket: Bucket = ResourceBuilder::new()
		.metadata("name", "HelloToken")
                .metadata("symbol", "HT")
                .new_token_fixed(1000);
    
    	// Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
	Self {
          sample_vault: Vault::with_bucket(my_bucket)
	}
	.instantiate()
}
```

Aliquam non nisi nec dolor mollis mollis dignissim vel eros. Donec maximus varius diam, vel mollis dui placerat quis. Proin non risus eget turpis commodo blandit sit amet at velit. Etiam sapien urna, dictum a enim vitae, tincidunt sollicitudin diam. Nulla ac massa lobortis, finibus magna dignissim, eleifend ligula. Duis vestibulum ullamcorper augue, vel sagittis tortor ornare maximus. Duis suscipit molestie lacus, a iaculis erat venenatis vitae. Praesent semper auctor scelerisque. Nam ultrices libero in consectetur convallis.
