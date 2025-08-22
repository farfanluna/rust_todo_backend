# Stage 1: Build the React application
FROM node:18 AS build

WORKDIR /app

# Copy package.json and package-lock.json
COPY package*.json ./

# Install dependencies
RUN npm install --legacy-peer-deps

# Copy the rest of the application source code
COPY . .

# Build the application
RUN npm run build

# Stage 2: Serve the application with Nginx
FROM nginx:1.23-alpine

# Copy the build output from the build stage
COPY --from=build /app/build /usr/share/nginx/html

# Copy the nginx configuration file
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Expose port 3001 for Nginx
EXPOSE 3001

# Start Nginx
CMD ["nginx", "-g", "daemon off;"]
