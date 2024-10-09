# ot-config

```bash
docker run -ti -v ./:/tpl opentalk/cforge:latest sh
```

## fpoop

```bash
cat  values.yml | tpl --file controller.toml.tpl -d yaml
```

yq -nM 'load("keycloak/values.yaml") * load("values.yaml")' | tpl -d yaml --file keycloak/templates/keycloak.conf.tpl > keycloak/config/keycloak.conf
