//! PDF de prueba de N páginas A4 con texto y algo de gráfico vectorial.

pub fn generate(pages: usize) -> Vec<u8> {
    let mut objects: Vec<Vec<u8>> = vec![Vec::new(), Vec::new()]; // catálogo, páginas
    objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec());
    let mut kids = Vec::with_capacity(pages);
    for i in 0..pages {
        let mut c = String::new();
        c.push_str(&format!(
            "BT /F1 20 Tf 60 780 Td (Demi - pagina {} de {pages}) Tj ET\n",
            i + 1
        ));
        for line in 0..32 {
            c.push_str(&format!(
                "BT /F1 11 Tf 60 {} Td (Linea {} de la pagina {}: texto de relleno para medir el render y la seleccion.) Tj ET\n",
                740 - line * 20,
                line + 1,
                i + 1
            ));
        }
        c.push_str(&format!(
            "0.17 0.37 0.38 RG 2 w 60 60 m 535 60 l S 0.89 0.82 0.76 rg {} 80 120 40 re f\n",
            60 + (i % 10) * 40
        ));
        objects.push(format!("<< /Length {} >>\nstream\n{c}endstream", c.len()).into_bytes());
        let content = objects.len();
        objects.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] \
                 /Resources << /Font << /F1 3 0 R >> >> /Contents {content} 0 R >>"
            )
            .into_bytes(),
        );
        kids.push(format!("{} 0 R", objects.len()));
    }
    objects[0] = b"<< /Type /Catalog /Pages 2 0 R >>".to_vec();
    objects[1] = format!(
        "<< /Type /Pages /Kids [{}] /Count {pages} >>",
        kids.join(" ")
    )
    .into_bytes();

    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len());
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    let n = objects.len() + 1;
    out.extend_from_slice(format!("xref\n0 {n}\n0000000000 65535 f \n").as_bytes());
    for off in offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!("trailer\n<< /Size {n} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
    );
    out
}
