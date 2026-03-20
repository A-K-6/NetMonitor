# Task 011: Geo-IP & ASN Mapping

**Status:** Proposed
**Phase:** 3 (Advanced)
**Objective:** Map destination IP addresses to countries and organizations (e.g., `142.250.190.46` -> `United States, Google LLC`).

## 1. Research & Strategy
- [x] **Database Choice:** Use a lightweight, embedded Geo-IP database (like `maxminddb` or a comparable provider) to keep the "Unified Binary" promise.
- [x] **Data Sourcing:** Research how to bundle the database (or a subset of it) into the binary using `include_bytes!`.
- [x] **Data Model:** Update `ConnectionInfo` to include `country_code` and `isp_name` (ASN).

## 2. Implementation (Userspace)
- [x] **Geo-IP Integration:** Add `maxminddb` (or similar) to `Cargo.toml`.
- [x] **Resource Embedding:** Use `include_bytes!` to bundle a lightweight GeoLite2 database (if licensing permits) or provide an easy way for users to download it.
- [x] **UI Update:** Add a "Location/ISP" column to the Connection Table in the Detail View.

## 3. Verification
- [x] **Functional Test:** Verify that connections to known international servers (e.g., `bbc.co.uk`) show the correct country code.
- [x] **Binary Size Check:** Measure how much the embedded database adds to the final executable size.

## 4. Documentation
- [x] Update `PRD.md` with instructions for updating the database.
- [x] Log completion in `devlog.md`.
