extern crate chrono;
use chrono::offset::Utc;
use chrono::{
    Duration,
    NaiveDateTime,
};

/// Response is an object returned from "successful" (as in no *errors*) scraplirs driver
/// operations.
#[allow(dead_code)]
pub struct Response {
    /// The host(name) of the device being interacted with.
    pub host: String,
    /// The port of the device being interacted with.
    pub port: u16,
    /// The actual input sent to the device.
    pub input: String,
    /// "Raw" (bytes) output of the operation represented by this `Response`.
    pub raw_result: Vec<u8>,
    /// String output of the output of the operation represented by this `Response`.
    pub result: String,
    /// Starting time of the operation represented by this `Response`.
    pub start_time: NaiveDateTime,
    /// Ending time of the operation represented by this `Response`.
    pub end_time: NaiveDateTime,
    /// Total time the operation represented by this `Response` took.
    pub elapsed_time: Duration,
    /// A list of strings that, if seen in an output, indicate that the originating input/command
    /// "failed".
    pub failed_when_contains: Vec<String>,
    /// Indicates if the operation was a success or failure. Failure in this case means we saw some
    /// `failed_when_contains` output in the response, *not* that there was an unrecoverable error.
    /// The latter case would result in an error being returned not a `Response` object.
    pub failed: bool,
}

impl Response {
    /// Initializes a new `Response` object.
    #[must_use]
    pub fn new(
        input: &str,
        host: &str,
        port: u16,
        failed_when_contains: Vec<String>,
    ) -> Self {
        Self {
            host: host.to_owned(),
            port,
            input: input.to_owned(),
            raw_result: vec![],
            result: String::new(),
            start_time: Utc::now().naive_utc(),
            end_time: Utc::now().naive_utc(),
            elapsed_time: Duration::zero(),
            failed_when_contains,
            failed: true,
        }
    }

    /// Record the result of an operation.
    ///
    /// # Panics
    ///
    /// Can panic if there is invalid utf-8 in the bytes in `b`.
    #[allow(clippy::expect_used)]
    pub fn record(
        &mut self,
        b: Vec<u8>,
    ) {
        self.end_time = Utc::now().naive_utc();

        self.elapsed_time = self.end_time - self.start_time;

        self.raw_result = b.clone();
        self.result = String::from_utf8(b).expect("invalid utf-8 in result");

        let mut is_failed: bool = false;

        for failed_when_contains_item in self.failed_when_contains.clone() {
            if !self.result.contains(&failed_when_contains_item) {
                continue;
            }

            is_failed = true;

            break;
        }

        if !is_failed {
            self.failed = false;
        }
    }
}

/// Response is an object returned from "successful" (as in no *errors*) scraplirs driver "multi"
/// operation -- that is a plural operation like `send_commands` or `send_configs` -- it holds the
/// individual `Response` objects for all steps/operations of the parent operation.
#[allow(clippy::module_name_repetitions)]
pub struct MultiResponse {
    /// The host(name) of the device being interacted with.
    pub host: String,
    /// Starting time of the operation represented by this `Response`.
    pub start_time: NaiveDateTime,
    /// Ending time of the operation represented by this `Response`.
    pub end_time: NaiveDateTime,
    /// Total time the operation represented by this `Response` took.
    pub elapsed_time: Duration,
    /// Vec of the individual responses that make up the "multi" response.
    pub responses: Vec<Response>,
    /// Indicates if the operation was a success or failure. Failure in this case means we saw some
    /// `failed_when_contains` output in the response, *not* that there was an unrecoverable error.
    /// The latter case would result in an error being returned not a `Response` object.
    pub failed: bool,
}

impl MultiResponse {
    /// Initializes a new `MultiResponse` object.
    #[must_use]
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_owned(),
            start_time: Utc::now().naive_utc(),
            end_time: Utc::now().naive_utc(),
            elapsed_time: Duration::zero(),
            responses: vec![],
            failed: false,
        }
    }

    /// Appends a response to the `MultiResponse` object.
    pub fn record_response(
        &mut self,
        response: Response,
    ) {
        self.end_time = Utc::now().naive_utc();

        self.elapsed_time = self.end_time - self.start_time;

        if response.failed {
            self.failed = true;
        }

        self.responses.push(response);
    }
}
