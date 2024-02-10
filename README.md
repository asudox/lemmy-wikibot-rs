# lemmy-wikibot-rs
A [Lemmy](https://join-lemmy.org/) bot written in Rust to summarize wikipedia articles and reply to them.

## How to run locally
1. Clone the repository: `git clone https://github.com/Asudox/lemmy-wikibot-rs.git`
    - Install Rust if you haven't yet: https://www.rust-lang.org/tools/install

3. Compile it with cargo: `cargo build --release`
4. Edit the `.env` file according to this table:

| Key                      | Description                                                          |
|--------------------------|----------------------------------------------------------------------|
| LEMMY_USERNAME_OR_EMAIL  | The username or email of the lemmy bot                               |
| LEMMY_PASSWORD           | The password of the lemmy bot                                        |
| LEMMY_INSTANCE           | The domain name of the lemmy instance where the bot is registered at |
| SENTENCE_REDUCTION_LIMIT | The sentence reduction limit (recommended value: 5)                  |

5. Run the bot: `cargo run --release`


## TODO
- [ ] Multiple wikipedia link support
- [ ] Better error handling (?)
- [ ] Fix wikipedia section support
- [ ] Add wikipedia search functionality

## License
This project is licensed under the [GNU Affero General Public License](https://www.gnu.org/licenses/agpl-3.0.html).
