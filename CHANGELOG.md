# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a
Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.3] - 2024-12-17

-  feat(dockerfile): Make workdir read/writable by root group for k8s
-  feat: Add option to configure via ENV variables (FAMEDLY_BDR)
-  bump: Update dependencies

## [0.3.2] - 2024-05-02

- Aggregate stats hourly instead of daily

## [0.3.1] - 2024-04-22

- Prepare sqlx queries for release builds

## [0.3.0] - 2024-04-22

- Aggregated stats by context
- Remove dependabot configuration

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
