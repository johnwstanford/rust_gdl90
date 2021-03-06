
pub fn lat_lon_dist_nm(phi1_deg:f32, lam1_deg:f32, phi2_deg:f32, lam2_deg:f32) -> f32 {
	let phi1:f32 = phi1_deg * crate::RAD_PER_DEG;
	let phi2:f32 = phi2_deg * crate::RAD_PER_DEG;
	let dphi:f32 = phi2 - phi1;
	let dlam:f32 = (lam2_deg - lam1_deg) * crate::RAD_PER_DEG;
	let a:f32 = (0.5 * dphi).sin().powi(2) + phi1.cos()*phi2.cos()*(0.5*dlam).sin().powi(2);
	let c:f32 = 2.0 * a.sqrt().atan2((1.0-a).sqrt());

	(crate::R * c) / crate::METERS_PER_NM
}
