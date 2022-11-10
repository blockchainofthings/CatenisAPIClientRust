# Changelog

## [3.0.0] - 2022-11-10

### Breaking changes
- The data structure returned by the *retrieve_asset_info* method of the CatenisClient object has been updated to
  include the new field *is_non_fungible* as per the new behavior of version 0.12 of the Catenis API.
- The data structure returned by the *retrieve_asset_issuance_history* method of the CatenisClient object has been
  updated to account for the two different issuance event types (for regular and for non-fungible assets) that are now
  returned as per the new behavior of version 0.12 of the Catenis API.
- The data structure returned by the *list_permission_events* method of the CatenisClient object has been updated to
  account for the new permission events introduced by version 0.12 of the Catenis API.
- The data structure returned by the *list_notification_events* method of the CatenisClient object and the data
  structure used for processing notification messages (*NotificationMessage*) have been updated to account for the new
  notification events introduced by version 0.12 of the Catenis API.
- Asynchronous processing now requires version 1 of the Tokio runtime.

### New features
- Added support for changes introduced by version 0.12 of the Catenis API: new non-fungible assets feature, including
  the new API methods Issue Non-Fungible Asset, Reissue Non-Fungible Asset, Retrieve Non-Fungible Asset Issuance
  Progress, Retrieve Non-Fungible Token, Retrieve Non-Fungible Token Retrieval Progress, Transfer Non-Fungible Token,
  Retrieve Non-Fungible Token Transfer Progress.

## [2.0.1] - 2021-09-02

### Fixes
- Fix documentation to reference the latest version of the Catenis API client library.

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