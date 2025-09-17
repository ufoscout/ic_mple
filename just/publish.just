
# Publishes all artifacts
[group('publish')]
publish_all: publish_utils publish_auth publish_pocket_ic publish_client publish_log


# Publishes ic_mple_auth
[group('publish')]
publish_auth:
  cargo publish -p ic_mple_auth
  

# Publishes ic_mple_client
[group('publish')]
publish_client:
  cargo publish -p ic_mple_client


# Publishes ic_mple_log
[group('publish')]
publish_log:
  cargo publish -p ic_mple_log


# Publishes ic_mple_pocket_ic
[group('publish')]
publish_pocket_ic:
  cargo publish -p ic_mple_pocket_ic


# Publishes ic_mple_pocket_ic
[group('publish')]
publish_utils:
  cargo publish -p ic_mple_utils