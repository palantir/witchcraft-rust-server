// Copyright 2022 Palantir Technologies, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! MDC keys managed by Witchcraft.
//!
//! All keys in the `witchcraft_log::mdc` will be included as parameters in service logs emitted
//! by Witchcraft. However, there are some keys that are treated specially.
//!
//! These keys are set automatically by Witchcraft and will be written to specific keys in the log
//! message rather than the generic params map.

/// The safe MDC key storing the value for the `uid` field in service logs.
pub const UID_KEY: &str = "\0witchcraft-uid";
/// The safe MDC key storing the value for the `sid` field in service logs.
pub const SID_KEY: &str = "\0witchcraft-sid";
/// The safe MDC key storing the value for the `tokenId` field in service logs.
pub const TOKEN_ID_KEY: &str = "\0witchcraft-token-id";
/// The safe MDC key storing the value for the `orgId` field in service logs.
pub const ORG_ID_KEY: &str = "\0witchcraft-org-id";
/// The safe MDC key storing the value for the `traceId` field in service logs.
pub const TRACE_ID_KEY: &str = "\0witchcraft-trace-id";
