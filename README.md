# Barad-dûr

Another matrix phone-home stats collector.

## Requirements

- TLS, so that servers can reach you securely.
- PostgreSQL, used for storing stats.

## Installation

You can either use [`cargo`](https://doc.rust-lang.org/cargo/) to install
Barad-dûr on your machine locally with `cargo install --git
https://gitlab.com/famedly/company/devops/services/barad-dur.git`, or deploy it
in a container, with images available from
`registry.gitlab.com/famedly/company/devops/services/barad-dur`.

## Usage

### Configuration

For configuration options look into `config.sample.yaml`.

### Importing existing data from panopticon

Barad-dûr has import scripts for panopticon, which you can find in `misc/panopticon-import`, together with usage instructions.

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

[AGPL-3.0-only](https://choosealicense.com/licenses/agpl-3.0/)

## Authors

- Shekhinah Memmel (she@khinah.xyz)
- Jan Christian Grünhage (jan.christian@gruenhage.xyz)
