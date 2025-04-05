use actix_multipart::Multipart;
use actix_web::{post, web, App, Error, HttpResponse, HttpServer};
use futures::{StreamExt, TryStreamExt};
use std::fs;
use std::io::Write;
use std::process::Command;
use uuid::Uuid;

#[post("/process")]
async fn process_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // Parcourt chaque champ du multipart
    while let Ok(Some(mut field)) = payload.try_next().await {
        println!("test");
        // On vérifie que le champ s'appelle "file"
        if let content_disposition = field.content_disposition() {
            if let Some(name) = content_disposition.get_name() {
                if name == "file" {
                    dbg!("name = file");
                    // Utilise le nom de fichier original s'il est présent, sinon génère un UUID
                    let filename = if let Some(fname) = content_disposition.get_filename() {
                        sanitize_filename::sanitize(fname)
                    } else {
                        Uuid::new_v4().to_string()
                    };

                    // Crée un dossier temporaire (./tmp) et définit le chemin de sauvegarde
                    fs::create_dir_all("./tmp").unwrap();
                    let temp_filepath = format!("./tmp/{}", filename);
                    let mut f = match fs::File::create(&temp_filepath) {
                        Ok(f) => f,
                        Err(e) => {
                            println!("Erreur lors de la création du fichier: {}", e);
                            return Ok(HttpResponse::InternalServerError().body("Erreur lors de l'écriture du fichier"));
                        }
                    };

                    // Écrit le contenu du fichier uploadé
                    while let Some(chunk) = field.next().await {
                        dbg!("ecriture");
                        let data = chunk
                            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
                        f.write_all(&data)
                            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
                    }
                    println!("i m there too");
                    
                    let packed_filepath = format!("{}.packed", temp_filepath);
let output = Command::new("./rpack")
    .arg(&temp_filepath)
    .arg(&packed_filepath)
    .output();

match output {
    Ok(out) => {
        println!("rPack stdout: {}", String::from_utf8_lossy(&out.stdout));
        println!("rPack stderr: {}", String::from_utf8_lossy(&out.stderr));
        println!("rPack status: {}", out.status);

        if !out.status.success() {
            return Ok(HttpResponse::InternalServerError().body("Échec de rPack"));
        }
    }
    Err(e) => {
        eprintln!("Erreur d’exécution de rPack: {:?}", e); // <= ça capture ENFIN l’erreur
        return Ok(HttpResponse::InternalServerError().body("Erreur exécution rPack"));
    }
}

                    // Appel de l'exécutable rPack avec le fichier temporaire en argument
                    // Le fichier packé est supposé être nommé "<temp_filepath>.packed"
                    let packed_data = fs::read(&packed_filepath)
                        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

                    // Optionnel : nettoyage des fichiers temporaires
                    //let _ = fs::remove_file(&temp_filepath);
                    //let _ = fs::remove_file(&packed_filepath);

                    // Retourne le fichier packé dans la réponse avec le bon nom pour téléchargement
                    let download_filename = format!("{}.packed", filename);
                    return Ok(HttpResponse::Ok()
                        .content_type("application/octet-stream")
                        .append_header((
                            "Content-Disposition",
                            format!("attachment; filename=\"{}\"", download_filename),
                        ))
                        .body(packed_data));
                }
            }
        }
    }
    Ok(HttpResponse::BadRequest().body("No file found"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server on 0.0.0.0:8080");
    println!("Utilisateur courant : {:?}", std::env::var("USER"));
    HttpServer::new(|| App::new().service(process_file))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}

