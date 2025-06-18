use crate::error::AppError;
use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;
use star_constellation::api::server;

#[derive(Deserialize, Clone)]
pub struct MeasureRequest {
    pub data: Vec<u8>,
    pub epoch: u8,
}

pub async fn process_instances_randomness(
    req: web::Json<MeasureRequest>,
) -> Result<impl Responder, AppError> {
    let msg = req.data.clone();
    let threshold = 2;
    let epoch = 1;

    // เตรียม measurement จากข้อความ
    let agg_res = server::aggregate(&[msg], threshold, epoch, 2);

    Ok(HttpResponse::Ok().body(format!(
        "Random data points len: {}",
        agg_res.outputs().len()
    )))
}
pub async fn process_instances_info() -> Result<impl Responder, AppError> {
    let public_key = "public_key";

    Ok(HttpResponse::Ok().body(format!(
        "public_key : {}",
        public_key
    )))
}
#[cfg(test)]
mod tests {
    use super::*;
    use star_constellation::api::client;
    use star_constellation::randomness::testing::{
        LocalFetcher, LocalFetcherResponse, PPOPRF_SERVER,
    };

    fn get_eval_output_slice_vecs(resp: &LocalFetcherResponse) -> (Vec<&[u8]>, Vec<&[u8]>) {
        (
            resp.serialized_points
                .iter()
                .map(|v| v.as_slice())
                .collect(),
            resp.serialized_proofs
                .iter()
                .map(|v| v.as_slice())
                .collect(),
        )
    }
    #[test]
    fn basic_test() {
        // สร้างตัวอย่าง MeasureRequest
        let epoch = 0;
        let threshold = 1;
        let measurement = vec!["hello".as_bytes().to_vec(), "world".as_bytes().to_vec()];
        let random_fetcher = LocalFetcher::new();
        let aux = "added_data".as_bytes().to_vec();
        let rrs = client::prepare_measurement(&measurement, epoch).unwrap();
        let req = client::construct_randomness_request(&rrs);

        let req_slice_vec: Vec<&[u8]> = req.iter().map(|v| v.as_slice()).collect();
        let resp = random_fetcher.eval(&req_slice_vec, epoch).unwrap();
        let (points_slice_vec, proofs_slice_vec) = get_eval_output_slice_vecs(&resp);

        let msg = client::construct_message(
            &points_slice_vec,
            Some(&proofs_slice_vec),
            &rrs,
            &Some(PPOPRF_SERVER.get_public_key()),
            &aux,
            threshold,
        )
        .unwrap();

        // เรียกใช้ aggregate
        let agg_res = server::aggregate(&[msg], threshold, epoch, 2);
        let outputs = agg_res.outputs();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].value(), vec!["world"]);
        assert_eq!(outputs[0].auxiliary_data(), vec![aux]);
        assert_eq!(agg_res.num_recovery_errors(), 0);
        assert_eq!(agg_res.num_serde_errors(), 0);
    }
    #[test]
    fn incompatible_epoch() {
        let c_epoch = 0u8;
        let threshold = 1;
        let measurement = vec!["hello".as_bytes().to_vec(), "world".as_bytes().to_vec()];
        let random_fetcher = LocalFetcher::new();
        let rrs = client::prepare_measurement(&measurement, c_epoch).unwrap();
        let req = client::construct_randomness_request(&rrs);

        let req_slice_vec: Vec<&[u8]> = req.iter().map(|v| v.as_slice()).collect();
        let resp = random_fetcher.eval(&req_slice_vec, c_epoch).unwrap();
        let (points_slice_vec, proofs_slice_vec) = get_eval_output_slice_vecs(&resp);

        let msg = client::construct_message(
            &points_slice_vec,
            Some(&proofs_slice_vec),
            &rrs,
            &Some(PPOPRF_SERVER.get_public_key()),
            &[],
            threshold,
        )
        .unwrap();
        let agg_res = server::aggregate(&[msg], threshold, 1u8, 2);
        assert_eq!(agg_res.num_recovery_errors(), 1);
        assert_eq!(agg_res.outputs().len(), 0);
    }
    #[test]
    fn incompatible_threshold() {
        let epoch = 0u8;
        let threshold = 3;
        let measurement = vec!["hello".as_bytes().to_vec(), "world".as_bytes().to_vec()];
        let random_fetcher = LocalFetcher::new();
        let messages: Vec<Vec<u8>> = (0..threshold - 1)
            .map(|_| {
                let rrs = client::prepare_measurement(&measurement, epoch).unwrap();
                let req = client::construct_randomness_request(&rrs);
                // let req = client::construct_message();

                let req_slice_vec: Vec<&[u8]> = req.iter().map(|v| v.as_slice()).collect();
                let resp = random_fetcher.eval(&req_slice_vec, epoch).unwrap();
                let (points_slice_vec, proofs_slice_vec) = get_eval_output_slice_vecs(&resp);

                client::construct_message(
                    &points_slice_vec,
                    Some(&proofs_slice_vec),
                    &rrs,
                    &Some(PPOPRF_SERVER.get_public_key()),
                    &[],
                    threshold,
                )
                .unwrap()
            })
            .collect();
        let agg_res = server::aggregate(&messages, threshold - 1, epoch, 2);
        assert_eq!(agg_res.num_recovery_errors(), 2);
        assert_eq!(agg_res.outputs().len(), 0);
    }
}
