# seaorm-basics

## Setup

```sh
docker run --name pgdev -e POSTGRES_PASSWORD=secret -p 5432:5432 -d postgres:16
cd 11-intro-to-databases/seaorm-basics
cp .env.example .env            # edit password/db name
export DATABASE_URL=postgresql://postgres:secret@localhost:5433/seaorm
```
## Generate entities

sea-orm-cli generate entity -u postgres://user:password@localhost/your_database_name -o src/entities

sea-orm-cli generate entity -u postgres://postgres:secret@localhost:5433/sea_test -o src/entities

## Migration

sea-orm-cli migrate init

