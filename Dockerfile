# Multi-stage build: build frontend, serve with nginx
FROM node:20-alpine AS builder
WORKDIR /app
COPY apps/frontend/package.json apps/frontend/pnpm-lock.yaml* ./
RUN npm install -g pnpm && pnpm install --frozen-lockfile
COPY apps/frontend/ .
RUN pnpm run build

FROM nginx:alpine
COPY --from=builder /app/out /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
