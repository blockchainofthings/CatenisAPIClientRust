# Changelog

## [2.0.0] - 2021-09-02

### Breaking changes
- The methods *log_message* and *log_chunked_message* of the CatenisClient object have been merged into a single method
  named *log_message*.
- The methods *send_message* and *send_chunked_message* of the CatenisClient object have been merged into a single
  method named *send_message*.
- The data structure returned by the *list_asset_holders* method of the CatenisClient object has been updated to account
  for the new entry that reports the total asset amount that is currently migrated to foreign blockchains as per the new
  behavior of version 0.11 of the Catenis API.

### New features
- Added support for changes introduced by version 0.11 of the Catenis API: new asset export feature, including the new
  API methods Export Asset, Migrate Asset, Asset Export Outcome, Asset Migration Outcome, List Exported Assets, and
  List Asset Migrations.

## [1.0.1] - 2021-01-04

### New features
- Initial version of the library.