A lemmy bot written in Rust to summarize wikipedia articles and reply to them.

# How to run locally
1. Clone the repository: `git clone https://github.com/Asudox/lemmy-wikibot-rs.git`
    - Install Rust if you haven't yet: https://www.rust-lang.org/tools/install

3. Compile it with cargo: `cargo build --release`
4. Create a new .env file in the directory with the following elements:

| Key                      | Value                                                                |
|--------------------------|----------------------------------------------------------------------|
| LEMMY_USERNAME_OR_EMAIL  | The username or email of the lemmy bot                               |
| LEMMY_PASSWORD           | The password of the lemmy bot                                        |
| LEMMY_INSTANCE           | The domain name of the lemmy instance where the bot is registered at |
| LEMMY_COMMUNITY          | The LOCAL community name (without the c/ prefix)                     |
| SENTENCE_REDUCTION_LIMIT | The sentence reduction limit                                         |

5. Run the bot: `cargo run --release`


# TODO
- [ ] Multiple wikipedia link support
- [ ] Better error handling (?)
- [x] Opt-out functionality
- [ ] Fix wikipedia section support
