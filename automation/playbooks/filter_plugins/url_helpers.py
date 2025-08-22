# filter_plugins/url_helpers.py

def build_host(args):
    """
    Builds a host[:port] string from a tuple or list.
    args: (domain, port)
    """
    if isinstance(args, (tuple, list)):
        domain = args[0]
        port = args[1] if len(args) > 1 else None
    else:
        # if passed directly as a single variable
        domain = args
        port = None

    if not port or str(port) in ['80', '443']:
        return f"{domain}"
    return f"{domain}:{port}"


def build_url(args):
    """
    Builds a URL from a tuple or list.
    args: (protocol, domain, port)
    """
    if isinstance(args, (tuple, list)):
        protocol = args[0]
        domain = args[1]
        port = args[2] if len(args) > 2 else None
    else:
        raise ValueError("build_url expects a tuple or list (protocol, domain[, port])")

    if not port or str(port) in ['80', '443']:
        return f"{protocol}://{domain}"
    return f"{protocol}://{domain}:{port}"


def build_wildcard_host(args):
    """
    Builds a host[:port] string with a leading '*.' wildcard.
    args: (domain, port)
    """
    if isinstance(args, (tuple, list)):
        domain = args[0]
        port = args[1] if len(args) > 1 else None
    else:
        domain = args
        port = None

    host = f"*.{domain}"

    if not port or str(port) in ['80', '443']:
        return host
    return f"{host}:{port}"


def build_wildcard_url(args):
    """
    Builds a URL where the host has a leading '*.' wildcard.
    args: (protocol, domain, port)
    """
    if isinstance(args, (tuple, list)):
        protocol = args[0]
        domain = args[1]
        port = args[2] if len(args) > 2 else None
    else:
        raise ValueError("build_wildcard_url expects a tuple or list (protocol, domain[, port])")

    host = f"*.{domain}"

    if not port or str(port) in ['80', '443']:
        return f"{protocol}://{host}"
    return f"{protocol}://{host}:{port}"


def build_subdomain_host(args):
    """
    Builds a host[:port] string with a subdomain.
    args: (subdomain, domain, port)
    """
    if isinstance(args, (tuple, list)):
        subdomain = args[0] or ''
        domain = args[1]
        port = args[2] if len(args) > 2 else None
    else:
        raise ValueError("build_subdomain_host expects a tuple or list (subdomain, domain[, port])")

    if not domain:
        raise ValueError("Domain must not be empty")

    host = f"{subdomain}.{domain}" if subdomain else domain
    if not port or str(port) in ['80', '443']:
        return host
    return f"{host}:{port}"


def build_subdomain_url(args):
    """
    Builds a URL with a subdomain.
    args: (protocol, subdomain, domain, port)
    """
    if isinstance(args, (tuple, list)):
        protocol = args[0] or 'http'
        subdomain = args[1] or ''
        domain = args[2]
        port = args[3] if len(args) > 3 else None
    else:
        raise ValueError("build_subdomain_url expects a tuple or list (protocol, subdomain, domain[, port])")

    if not domain:
        raise ValueError("Domain must not be empty")

    host = build_subdomain_host((subdomain, domain, port))
    return f"{protocol}://{host}"


class FilterModule(object):
    def filters(self):
        return {
            'build_host': build_host,
            'build_url': build_url,
            'build_wildcard_host': build_wildcard_host,
            'build_wildcard_url': build_wildcard_url,
            'build_subdomain_host': build_subdomain_host,
            'build_subdomain_url': build_subdomain_url,
        }
