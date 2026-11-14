# syntax=docker/dockerfile:1
# check=error=true

# The production image. Kamal builds it (see config/deploy.yml); to try it by hand:
#   docker build -t piladerflice .
#   docker run -d -p 80:80 -e SECRET_KEY_BASE=<bin/rails secret> -v piladerflice_storage:/rails/storage piladerflice

ARG RUBY_VERSION=3.3.6
FROM docker.io/library/ruby:$RUBY_VERSION-slim AS base

WORKDIR /rails

# Runtime packages: jemalloc (less memory), sqlite3 (the CLI, for backups and
# poking at the database), curl (health checks).
RUN apt-get update -qq && \
    apt-get install --no-install-recommends -y curl libjemalloc2 sqlite3 && \
    ln -s /usr/lib/$(uname -m)-linux-gnu/libjemalloc.so.2 /usr/local/lib/libjemalloc.so && \
    rm -rf /var/lib/apt/lists /var/cache/apt/archives

ENV RAILS_ENV="production" \
    BUNDLE_DEPLOYMENT="1" \
    BUNDLE_PATH="/usr/local/bundle" \
    BUNDLE_WITHOUT="development:test" \
    LD_PRELOAD="/usr/local/lib/libjemalloc.so" \
    TZ="Europe/Prague"

# Build stage: compilers for gems with native extensions, then thrown away.
FROM base AS build

RUN apt-get update -qq && \
    apt-get install --no-install-recommends -y build-essential git libyaml-dev pkg-config && \
    rm -rf /var/lib/apt/lists /var/cache/apt/archives

COPY Gemfile Gemfile.lock ./
RUN bundle install && \
    rm -rf ~/.bundle/ "${BUNDLE_PATH}"/ruby/*/cache "${BUNDLE_PATH}"/ruby/*/bundler/gems/*/.git

COPY . .

# Assets are fingerprinted and served with a one-year cache header. A dummy
# secret is enough for precompiling; the real one comes from the server.
RUN SECRET_KEY_BASE_DUMMY=1 ruby bin/rails assets:precompile

# Final image: the app and its gems, run as an unprivileged user.
FROM base

RUN groupadd --system --gid 1000 rails && \
    useradd rails --uid 1000 --gid 1000 --create-home --shell /bin/bash
USER 1000:1000

COPY --chown=rails:rails --from=build "${BUNDLE_PATH}" "${BUNDLE_PATH}"
COPY --chown=rails:rails --from=build /rails /rails

# Creates/migrates the databases before the server starts.
ENTRYPOINT ["bash", "/rails/bin/docker-entrypoint"]

# Kamal's proxy in front does HTTPS and talks to Puma on port 80.
ENV PORT="80"
EXPOSE 80
# (bin/rails is called through ruby: its first line points at ruby.exe, for Windows.)
CMD ["ruby", "bin/rails", "server", "-b", "0.0.0.0"]
