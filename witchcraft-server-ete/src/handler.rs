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
use crate::conjure::TestService;
use conjure_error::{Error, InvalidArgument};
use conjure_http::server::WriteBody;
use std::io::Write;
use std::thread;
use std::time::Duration;
use witchcraft_server::blocking::ResponseWriter;

pub struct TestResource;

impl TestService<ResponseWriter> for TestResource {
    type SlowBodyBody = SlowBodyBody;

    fn safe_params(
        &self,
        safe_path: String,
        unsafe_path: String,
        safe_query: String,
        unsafe_query: String,
        safe_header: String,
        unsafe_header: String,
    ) -> Result<(), Error> {
        assert_eq!(safe_path, "expected safe path");
        assert_eq!(unsafe_path, "expected unsafe path");
        assert_eq!(safe_query, "expected safe query");
        assert_eq!(unsafe_query, "expected unsafe query");
        assert_eq!(safe_header, "expected safe header");
        assert_eq!(unsafe_header, "expected unsafe header");

        Ok(())
    }

    fn slow_headers(&self, delay_millis: i32) -> Result<(), Error> {
        thread::sleep(Duration::from_millis(delay_millis as u64));
        Ok(())
    }

    fn slow_body(&self, delay_millis: i32) -> Result<SlowBodyBody, Error> {
        Ok(SlowBodyBody(Duration::from_millis(delay_millis as u64)))
    }
}

pub struct SlowBodyBody(Duration);

impl WriteBody<ResponseWriter> for SlowBodyBody {
    fn write_body(self: Box<Self>, w: &mut ResponseWriter) -> Result<(), Error> {
        w.write_all(&[0])
            .map_err(|e| Error::service_safe(e, InvalidArgument::new()))?;
        w.flush()
            .map_err(|e| Error::service_safe(e, InvalidArgument::new()))?;
        thread::sleep(self.0);
        w.write_all(&[0])
            .map_err(|e| Error::service_safe(e, InvalidArgument::new()))?;

        Ok(())
    }
}
