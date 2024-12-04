# CKZiU CodeFest API

![Rust](https://img.shields.io/badge/Rust-🦀-orange?style=flat-square)

The backend API for **CKZiU CodeFest**, designed to support event management,
handle participants and store result efficiently.

---

## 🚀 Features

- **User Management**
  - Registration and login with JWT-based authentication
  - Restrict registration to only @ckziu.lodz.edu.pl
  - User roles (student, developer, admin etc.)
- **High Performance**
  - Built with **Rust lang** and **Warp** for speed and safety.
 
## 🛠️ Technologies

- **Framework**: Warp (Rust)
- **Database**: PostgreSQL (with SQLx)
- **Containerization**: Docker & Docker Compose
- **Authentication**: JWT
- **Environment Variables**: Configured via `.env`

## 🖥️ Installation & Setup

### 1. Clone the Repository
```bash
git clone https://github.com/Moderrek/ckziu-codefest-api.git
cd ckziu-codefest-api
```

### 2. Configure Environment Variables
Copy the `.env.template` file and set your environment variables:
```bash
cp .env.template .env
```

### 3. Run Locally
```bash
cargo run
```

The API will be available at: http://localhost:8000

# 🌐 API Endpoints

### 🧑‍💼 **User Management**
| Method | Endpoint                     | Description                                                |
|--------|-------------------------------|------------------------------------------------------------|
| `GET`  | `/v1/users?page=x`           | Retrieve paginated users.                                  |
| `GET`  | `/v1/users/{name}`           | Retrieve user data.                                        |
| `PATCH`| `/v1/users/{name}`           | Requires auth. Updates user data with given JSON body.     |

---

### 🔒 **Authentication**
| Method | Endpoint                     | Description                                                |
|--------|-------------------------------|------------------------------------------------------------|
| `GET`  | `/v1/auth/info`              | Gets current session data.                                 |
| `POST` | `/v1/auth/prelogin`          | Sends OTP code if needed.                                  |
| `POST` | `/v1/auth/otp`               | Verifies OTP code.                                         |
| `POST` | `/v1/auth/register`          | Registers new user using OTP.                              |
| `POST` | `/v1/auth/login/credentials` | Requires login and email to log in.                        |

---

### 👤 **Profile Management**
| Method | Endpoint                     | Description                                                |
|--------|-------------------------------|------------------------------------------------------------|
| `GET`  | `/v1/profile/{name}`         | Retrieve user data, projects, and posts.                   |
| `GET`  | `/v1/avatars/{name}`         | Retrieve user profile picture.                             |
| `POST` | `/v1/upload/avatar`          | Requires auth. Uploads a new avatar.                       |
| `POST` | `/v1/update/user/displayname`| Requires auth. Updates display name.                       |
| `POST` | `/v1/update/user/bio`        | Requires auth. Updates bio.                                |

---

### 📢 **News**
| Method | Endpoint                     | Description                                                |
|--------|-------------------------------|------------------------------------------------------------|
| `GET`  | `/v1/ckziu/news`             | Retrieve web scraped news from the school website.         |

---

### 🛠️ **Projects**
| Method   | Endpoint                                 | Description                                                |
|----------|------------------------------------------|------------------------------------------------------------|
| `GET`    | `/v1/projects`                          | Retrieve all project data with content.                    |
| `GET`    | `/v1/projects/{username}/{projectname}` | Maybe requires auth. Retrieve project data and content.    |
| `PATCH`  | `/v1/projects/{username}/{projectname}` | Requires auth. Updates project with JSON body.             |
| `DELETE` | `/v1/projects/{username}/{projectname}` | Requires auth. Deletes the entire project.                 |

---

### 🏆 **Contest Projects**
| Method | Endpoint                     | Description                                                |
|--------|-------------------------------|------------------------------------------------------------|
| `GET`  | `/v1/contestprojects`        | Retrieve all contest projects with votes (content excluded).|

---

### 📝 **Posts**
| Method   | Endpoint                     | Description                                                |
|----------|-------------------------------|------------------------------------------------------------|
| `GET`    | `/v1/posts`                 | Retrieve all posts.                                        |
| `POST`   | `/v1/posts`                 | Requires auth. Creates a post on your profile.             |
| `DELETE` | `/v1/posts/{id}`            | Requires auth. Deletes the post with the given ID.         |
| `GET`    | `/v1/posts/{id}/like`       | Requires auth. Likes the post.                             |
| `GET`    | `/v1/posts/{id}/unlike`     | Requires auth. If liked, unlikes the post.                 |

---
