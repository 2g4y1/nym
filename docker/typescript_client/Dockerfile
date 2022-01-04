FROM node:17-slim
RUN apt update && apt install -y netcat
COPY entrypoint.sh /entrypoint.sh
CMD /entrypoint.sh
