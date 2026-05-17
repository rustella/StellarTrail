FROM node:22-alpine AS builder
WORKDIR /app
ARG VITE_STELLARTRAIL_API_BASE_URL=https://api.example.invalid
ENV VITE_STELLARTRAIL_API_BASE_URL=${VITE_STELLARTRAIL_API_BASE_URL}
COPY package.json package-lock.json ./
COPY apps/web/package.json apps/web/package.json
COPY packages/api-client-ts/package.json packages/api-client-ts/package.json
COPY packages/shared-types/package.json packages/shared-types/package.json
RUN npm ci
COPY apps/web apps/web
COPY packages packages
RUN npm run build --workspace @stellartrail/web

FROM nginx:1.27-alpine AS runtime
COPY infra/production/web/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/apps/web/dist /usr/share/nginx/html
EXPOSE 80
