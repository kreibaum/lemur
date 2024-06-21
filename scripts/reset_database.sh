# Use Diesel CLI to drop the database and re-migrate it.

# Load environment variables
cd ..
source .env

# Drop the database
diesel database reset

# Run migrations
diesel migration run