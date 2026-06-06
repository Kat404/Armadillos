use crate::AppState;
use crate::layouts::main_layout::{PageContext, layout};
use crate::security::CsrfToken;
use axum::{
    Extension, Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar};
use maud::html;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormLogin {
    pub usuario: String,
    pub contrasena: String,
}

/// Handler GET `/login`: Muestra el formulario de inicio de sesión.
pub async fn pagina_login(
    Extension(csrf_token): Extension<CsrfToken>,
    jar: PrivateCookieJar,
) -> impl IntoResponse {
    // Si ya hay una sesión activa, redirigir directamente a /inventario
    if jar.get("operador_soldado_id").is_some() {
        return Redirect::to("/inventario").into_response();
    }

    let ctx = PageContext::new(
        "Armadillos - Inicio de Sesión",
        "Inicio de Sesión Militar",
        "Acceso seguro para el control de inventario y personal militar.",
        &csrf_token.0,
    );

    let html_content = html! {
        div class="row align-center justify-center" style="min-height: 70vh;" {
            div class="s12 m6 l4" {
                article class="round border medium-padding shadow" {
                    div class="row align-center justify-center margin-bottom" {
                        i class="extra primary-text" { "shield" }
                        h4 class="bold primary-text text-center no-margin" { " Armadillos" }
                    }
                    p class="text-secondary text-center margin-bottom" {
                        "Ingresa tus credenciales del Sistema de Administración Militar."
                    }

                    form action="/login" method="POST" class="column gap" {
                        div class="field label border round" {
                            input type="text" name="usuario" required="" autocomplete="username";
                            label { "Usuario Militar" }
                        }

                        div class="field label border round" {
                            input type="password" name="contrasena" required="" autocomplete="current-password";
                            label { "Contraseña" }
                        }

                        button type="submit" class="button round primary extend margin-top" {
                            i { "login" }
                            span { "Acceder al Sistema" }
                        }
                    }

                    div class="divider margin-top margin-bottom" {}
                    p class="caption text-secondary text-center no-margin" {
                        "Acceso restringido y monitoreado bajo el Código de Justicia Militar."
                    }
                }
            }
        }
    };

    layout(&ctx, html_content).into_response()
}

/// Handler POST `/login`: Procesa las credenciales e inyecta la cookie de sesión cifrada.
pub async fn procesar_login(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Form(payload): Form<FormLogin>,
) -> impl IntoResponse {
    let conn = match state.db.connect() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error de base de datos: {}", e),
            )
                .into_response();
        }
    };

    // Consultar el hash y el soldado_id
    let mut rows = match conn
        .query(
            "SELECT soldado_id, password_hash FROM credenciales_soldados WHERE usuario = ?1",
            (payload.usuario.trim().to_lowercase(),),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error de consulta: {}", e),
            )
                .into_response();
        }
    };

    let login_exito = if let Some(row) = rows.next().await.unwrap_or(None) {
        let soldado_id: i64 = row.get(0).unwrap_or(0);
        let password_hash: String = row.get(1).unwrap_or_default();

        use argon2::{Argon2, PasswordHash, PasswordVerifier};
        if let Ok(parsed_hash) = PasswordHash::new(&password_hash) {
            if Argon2::default()
                .verify_password(payload.contrasena.as_bytes(), &parsed_hash)
                .is_ok()
            {
                Some(soldado_id)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    match login_exito {
        Some(soldado_id) => {
            // Guardamos el ID en la cookie cifrada.
            // Directivas seguras: Secure, HttpOnly, SameSite=Strict, Path=/
            let cookie = Cookie::build(("operador_soldado_id", soldado_id.to_string()))
                .path("/")
                .http_only(true)
                .same_site(axum_extra::extract::cookie::SameSite::Strict)
                .build();

            let updated_jar = jar.add(cookie);
            (updated_jar, Redirect::to("/inventario")).into_response()
        }
        None => {
            // Si falla renderizamos la misma página pero con una alerta de error.
            let ctx = PageContext::new(
                "Armadillos - Inicio de Sesión",
                "Inicio de Sesión Militar",
                "Acceso seguro para el control de inventario y personal militar.",
                "",
            );

            let alerta_html = crate::components::alerta::alerta(
                "error",
                "ACCESO DENEGADO",
                "Usuario o contraseña incorrectos. Por favor verifique sus datos.",
            );

            let html_content = html! {
                div class="row align-center justify-center" style="min-height: 70vh;" {
                    div class="s12 m6 l4" {
                        (alerta_html)
                        article class="round border medium-padding shadow" {
                            div class="row align-center justify-center margin-bottom" {
                                i class="extra primary-text" { "shield" }
                                h4 class="bold primary-text text-center no-margin" { "Armadillos" }
                            }
                            p class="text-secondary text-center margin-bottom" {
                                "Ingresa tus credenciales del Sistema de Administración Militar."
                            }

                            form action="/login" method="POST" class="column gap" {
                                div class="field label border round" {
                                    input type="text" name="usuario" value=(payload.usuario) required="" autocomplete="username";
                                    label { "Usuario Militar" }
                                }

                                div class="field label border round" {
                                    input type="password" name="contrasena" required="" autocomplete="current-password";
                                    label { "Contraseña" }
                                }

                                button type="submit" class="button round primary extend margin-top" {
                                    i { "login" }
                                    span { "Acceder al Sistema" }
                                }
                            }

                            div class="divider margin-top margin-bottom" {}
                            p class="caption text-secondary text-center no-margin" {
                                "Acceso restringido y monitoreado bajo el Código de Justicia Militar."
                            }
                        }
                    }
                }
            };

            let response_markup = layout(&ctx, html_content);
            (StatusCode::UNAUTHORIZED, response_markup).into_response()
        }
    }
}

/// Handler POST `/logout`: Cierra la sesión activa.
pub async fn logout(jar: PrivateCookieJar) -> impl IntoResponse {
    let cookie = Cookie::build(("operador_soldado_id", ""))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .build();
    let updated_jar = jar.remove(cookie);
    (updated_jar, Redirect::to("/login")).into_response()
}
