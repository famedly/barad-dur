# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a
Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2024-03-21

- Add `GET` `/aggregated-stats` endpoint
- *(ci)* Correct syntax
- Move run function to lib
- Set health check and time zone
- Add github action
- Migrate to github
- Update to new reusable workflow

## [0.1.2] - 2023-04-19

Fix panic on logging.

## [0.1.1] - 2022-12-19

Updating dependencies, resolving several RUSTSEC advisories.

## [0.1.0] - 2021-10-28

Initial release of Barad-dûr. Comes with support for stats recording,
and aggregation.

A focus on maintainability means that this initial release already comes with:
- integration testing
- database schema migrations
- continuous integration
