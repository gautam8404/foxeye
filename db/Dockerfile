FROM postgres:16

RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    postgresql-server-dev-16


RUN git clone https://github.com/pgvector/pgvector /usr/src/pgvector

# Build and install pg_vector
RUN cd /usr/src/pgvector && make && make install

RUN apt-get remove -y build-essential git \
    && apt-get autoremove -y \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

CMD ["postgres"]


