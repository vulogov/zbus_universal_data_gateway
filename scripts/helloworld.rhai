fn filter(data) {
  true
}

fn transformation(data) {
  data.body.details.added_by_transformation = "Transformation routine been here";
  data
}
