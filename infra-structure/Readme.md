GitHub Copilot: To start Kafka using the updated docker-compose.yml file, follow these steps:

1. **Navigate to the directory containing the docker-compose.yml file**:
   ```bash
   cd /workspaces/gryphon-app/infra-structure
   ```

2. **Start the Kafka service**:
   Run the following command:
   ```bash
   docker-compose up -d
   ```

   This will start Kafka in detached mode.

3. **Verify Kafka is running**:
   Check the running containers:
   ```bash
   docker ps
   ```

   You should see a container for Kafka.

4. **Access Kafka logs (optional)**:
   If you want to view the logs for Kafka, use:
   ```bash
   docker-compose logs kafka
   ```

Let me know if you encounter any issues!