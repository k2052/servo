/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::TEST_PIPELINE_ID;
use http::StatusCode;
use http::header::{EXPIRES, HeaderValue};
use net::http_cache::HttpCache;
use net_traits::request::{Referrer, RequestBuilder};
use net_traits::response::{Response, ResponseBody};
use net_traits::{ResourceFetchTiming, ResourceTimingType};
use servo_url::ServoUrl;
use tokio::sync::mpsc::unbounded_channel as unbounded;

#[test]
fn test_refreshing_resource_sets_done_chan_the_appropriate_value() {
    let response_bodies = vec![
        ResponseBody::Receiving(vec![]),
        ResponseBody::Empty,
        ResponseBody::Done(vec![]),
    ];
    let url = ServoUrl::parse("https://servo.org").unwrap();
    let request = RequestBuilder::new(None, url.clone(), Referrer::NoReferrer)
        .pipeline_id(Some(TEST_PIPELINE_ID))
        .origin(url.origin())
        .build();
    let timing = ResourceFetchTiming::new(ResourceTimingType::Navigation);
    let mut response = Response::new(url.clone(), timing);
    // Expires header makes the response cacheable.
    response
        .headers
        .insert(EXPIRES, HeaderValue::from_str("-10").unwrap());
    let mut cache = HttpCache::default();
    response_bodies.iter().for_each(|body| {
        *response.body.lock().unwrap() = body.clone();
        // First, store the 'normal' response.
        cache.store(&request, &response);
        // Second, mutate the response into a 304 response, and refresh the stored one.
        response.status = StatusCode::NOT_MODIFIED.into();
        let (send, recv) = unbounded();
        let mut done_chan = Some((send, recv));
        let refreshed_response = cache.refresh(&request, response.clone(), &mut done_chan);
        // Ensure a resource was found, and refreshed.
        assert!(refreshed_response.is_some());
        match body {
            ResponseBody::Receiving(_) => assert!(done_chan.is_some()),
            ResponseBody::Empty | ResponseBody::Done(_) => assert!(done_chan.is_none()),
        }
    })
}
