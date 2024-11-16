
# Publishes all artifacts
[group('publish')]
publish_all: publish_client publish_pocket_ic


# Publishes ic_mple_client
[group('publish')]
publish_client:
  cargo publish -p ic_mple_client


# Publishes ic_mple_pocket_ic
[group('publish')]
publish_pocket_ic:
  cargo publish -p ic_mple_pocket_ic