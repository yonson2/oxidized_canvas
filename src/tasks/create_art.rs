use loco_rs::prelude::*;

use crate::{
    common,
    models::{_entities::arts, arts::ArtParams},
    services::{
        ai::traits::{ImageGenerator, TextGenerator},
        service_provider::ServiceProvider,
    },
};

pub struct CreateArt;
#[async_trait]
impl Task for CreateArt {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "create_art".to_string(),
            detail: "Task generator".to_string(),
        }
    }
    async fn run(&self, ctx: &AppContext, _vars: &task::Vars) -> Result<()> {
        let settings =
            common::settings::Settings::from_json(&ctx.config.settings.clone().ok_or(0).unwrap())?;

        let img_gen = ServiceProvider::img_service(&settings.openai_key);
        let text_gen = ServiceProvider::txt_service(&settings.anthropic_key);

        let arts = arts::Model::find_n_random(&ctx.db, 10).await?;
        let image_generator_prompt = match arts.len() {
            x if x > 1 => gen_img_prompt(&arts),
            _ => IMAGE_PROMPT.replace("{{PROMPTS}}", SAMPLE_PROMPTS),
        };

        let Ok(prompt) = text_gen.generate(&image_generator_prompt).await else {
            return Err(loco_rs::errors::Error::Message("text_gen 1".to_string()));
        };
        println!("Prompt for image is: {prompt}");

        let title_generator_prompt = match arts.len() {
            x if x > 1 => gen_title_prompt(&prompt, &arts),
            _ => TITLE_PROMPT
                .replace("{{TITLES}}", SAMPLE_TITLES)
                .replace("{{DESCRIPTION}}", &prompt),
        };

        let (title, image) = (
            match text_gen.generate(&title_generator_prompt).await {
                Ok(t) => t,
                Err(_) => return Err(loco_rs::errors::Error::Message("text_gen 2".to_string())),
            },
            match img_gen.generate(&prompt).await {
                Ok(i) => i,
                Err(e) => {
                    println!("ERROR: {e}");
                    return Err(loco_rs::errors::Error::Message("img_gen 1".to_string()));
                }
            },
        );

        let art = arts::Model::create(
            &ctx.db,
            &ArtParams {
                image,
                prompt,
                title,
            },
        )
        .await?;

        println!("Created art: {} - {}", art.id, art.title);
        Ok(())
    }
}

fn gen_title_prompt(desc: &str, arts: &[arts::Model]) -> String {
    let titles = arts
        .iter()
        .map(|a| a.title.clone())
        .collect::<Vec<String>>()
        .join(", ");

    TITLE_PROMPT
        .replace("{{TITLES}}", &titles)
        .replace("{{DESCRIPTION}}", desc)
}

fn gen_img_prompt(arts: &[arts::Model]) -> String {
    let prompts = arts
        .iter()
        .enumerate()
        .map(|(i, a)| format!(" - prompt {i}: {}", a.title.clone()))
        .collect::<Vec<String>>()
        .join("\n");
    IMAGE_PROMPT.replace("{{PROMPTS}}", &prompts)
}

const SAMPLE_TITLES: &str = "Vivid Dreamscape: Dali-esque, Glowing Grove, Lily Luminescence, Geometric Genesis, Diminuendo Dusk";
const SAMPLE_PROMPTS: &str = "
 - prompt 1: A painting of a vivid, dreamy, surreal landscape, inspired by the works of Salvador Dali.
 - prompt 2: A digital composition reminiscent of Henri Rousseau's lush jungles, depicting a surreal nocturnal scene where bioluminescent plants and creatures create a symphony of light under the canopy. A tranquil azure pond, reflecting the subtle glow and intertwined vines forming natural arabesques, adds to the dreamlike atmosphere, while a hidden tiger's eyes glimmer with the wisdom of the wild.
 - prompt 3: A photographic interpretation of Monet's 'Water Lilies,' capturing the ephemeral beauty of a pond filled with luminous, blushing pink lily petals lightly dusted with morning dew, the surface broken only by the darting flash of elusive golden-orange koi and mirrored by a dappled canvas of sky, clouds, and overhanging willow branches mirrored on the calm water.
 - prompt 4: A modernist oil painting inspired by Pablo Picasso, showcasing an abstract geometric rendering of a flourishing garden, birthed through the symbiotic dance between botany and geometry. Sharp vertices of emerald-green leaves mingle with rounds of celestial-blue blooms. Crimson rays of a cubist sun uniquely fracture and reshape the landscape whilst sporadic golden-rust pebbles and sapphire rivulets conjugate to bring lucid texture to this spatial dialogue. Sprouting within this fertile collision of shape and form, a single orchid pink geometry expresses itself with defiant beauty elicited from the chaos.
 - prompt 5: A painterly photograph channeling the chiaroscuro intensity of Caravaggio's work to illustrate a modern street violinist lost in the emotion of a nocturne piece, with dramatic highlights cutting through the encroaching shadows of a forgotten alley.";

const TITLE_PROMPT: &str = "Create a captivating and imaginative title that goes beyond a literal interpretation.
Avoid using the artist's name, and feel free to incorporate clever wordplay or alliteration inspired by the description.
Here are some titles that you can take some inspiration from (never use these verbatim, they serve as inspiration for you to come up with something original): {{TITLES}}
The title should be no longer than 27 characters (Ideally it should be quite short, between two or three words) and evoke a sense of beauty, emotion, or intrigue.
For the following description (remember to give me *just* the title): {{DESCRIPTION}}.";

const IMAGE_PROMPT: &str = "Can you create a prompt to generate a compelling and highly artistic image using an AI system like DALL-E 2, MidJourney, StableDiffusion...? The emphasis should be on creativity, unique styles, and painterly or illustrative qualities rather than photorealism.
Important rules:
1. Try NOT to use any of these overused elements to maintain originality:
   - Butterflies or moths
   - Libraries or books
   - Lighthouses
   - Excessive mist/fog/ethereal scenes unless conceptually crucial
   - The words 'ethereal', 'whispers', 'ephemeral'
   - Simplistic surrealist transformations where X turns into Y without deeper meaning
   - Generic autumn scenes
   - Basic watercolors of nature (strive for more unique applications of watercolor)
2. Explore these less common subjects to inspire fresh visuals:
   - Urban life and city rhythms, focusing on unique perspectives or moods
   - Human emotions and intimate expressions, artistically rendered
   - Industrial and mechanical aesthetics, perhaps with an art deco or steampunk twist
   - Cultural ceremonies and traditions, depicted with vibrant detail and style
   - Scientific concepts and discoveries, visualized imaginatively
   - Historical moments and period-specific scenes, with attention to artistic interpretation of the era
   - Abstract geometrical compositions, exploring form, color, and texture
   - Architectural details and patterns, perhaps in an Escher-like or expressionistic style
   - Candid street scenes, captured with a painterly or illustrative feel
   - Still life with unexpected or symbolic objects, focusing on composition and light
   - Sports and movement, conveyed with dynamic lines and artistic flair
   - Fashion and textile details, emphasizing patterns, textures, and artistic representation
   - Food and culinary arts, presented as a work of art
   - Traditional crafts and artisanal work, highlighting the skill and beauty
   - Musical instruments and sound visualization, abstractly or synesthetically
   - Maritime and underwater scenes, focusing on artistic style rather than pure realism (avoiding overly mystical unless specifically requested for a concept)
   - Desert landscapes and arid environments, with unique color palettes or stylistic interpretations
   - Archaeological discoveries, imagined or reconstructed with an artistic touch
   - Modern technology, integrated into artistic compositions or critiqued visually
   - Dance and performance arts, capturing the energy and emotion through line and color
   - Weather phenomena, dramatically or abstractly represented
   - Markets and commerce, focusing on the human element and vibrant atmosphere
   - Transportation and vehicles, perhaps stylized or placed in imaginative settings
   - Wildlife in action, rendered with artistic expression, not just as a nature photograph
   - Medical and anatomical imagery, approached from an artistic or historical illustration perspective
   - Astronomical phenomena, artistically interpreted (avoiding generic dreamy space art)
   - Medieval or renaissance scenes, with a focus on the art styles of those periods or a modern reinterpretation
   - Construction and building processes, finding beauty in the unfinished or industrial
   - Religious and spiritual practices, depicted with respect and artistic depth
   - Military history, focusing on the human aspect or symbolic representation rather than glorification

3. Additional requirements for an Artistic Focus:
   - Prioritize artistic interpretations, painterly styles, and abstract or stylized visuals over photorealism. The goal is unique, imaginative artwork that evokes emotion.
   - Strongly encourage a rich variety of artistic mediums: oil painting (impasto, glazing), watercolor (wet-on-wet, dry brush), gouache, charcoal, ink wash (sumi-e), digital painting (concept art, matte painting), detailed illustration (pen and ink, cross-hatching), printmaking (linocut, etching, woodblock), collage, mixed media. Emphasize texture, brushwork, and medium-specific characteristics.
   - Actively incorporate a wide range of art movements: impressionism, post-impressionism, expressionism (German Expressionism, Abstract Expressionism), cubism (Analytical, Synthetic), fauvism, art nouveau, art deco, bauhaus, pop art, surrealism (if used thoughtfully and not generically), symbolism, futurism, constructivism, contemporary digital art styles, glitch art, generative art, folk art styles from various cultures. Explore beyond the most famous examples.
   - While photographic techniques or specific camera/lens effects (e.g., 'macro lens', 'long exposure', 'bokeh', 'fisheye perspective', 'shot on Portra 400 film') can be an inspiration or a descriptive element, the final generated image should lean towards a painterly, illustrative, or otherwise stylized artistic representation. Explicitly requesting 'a photograph of X' should be very rare and only if it serves a unique artistic concept that subverts typical photography.
   - Vary between different times of day (not just dawn/dusk) and explore unconventional, dramatic, or symbolic lighting (e.g., chiaroscuro, rim lighting, neon glow).
   - Use diverse and expressive color palettes. Consider color theory: complementary colors, analogous colors, triadic schemes, or even discordant colors for emotional impact. Don't be afraid of bold or unusual color choices.
   - When including human elements, focus on emotion, pose, gesture, and artistic representation rather than photorealistic portraiture. Figures can be stylized, elongated, or abstracted.
   - Consider different cultural perspectives and art traditions from around the world.
   - Explore both macro (close-up details with artistic focus) and micro (vast scenes with a sense of scale and composition) scales with an artistic eye.
   - When using artists as reference, prioritize lesser-known or contemporary artists, and those from diverse backgrounds and cultures, to foster originality and avoid direct imitation of overly famous styles unless reinterpreted uniquely.
   - Avoid overused artistic styles if they lead to generic results (e.g., generic dreamy surrealism, simplistic minimalism without conceptual depth). The aim is freshness and thoughtful execution.
   - Don't start prompts with generic phrases like 'A painting of X' or 'An image of Y'. Instead, describe the scene, the style, the emotion, the medium, and the composition directly and evocatively. For example, instead of 'A painting of a cat', try 'Impressionistic oil painting of a calico cat lounging in a sunbeam, dappled light, visible brushstrokes.'
   - Use specific technical terms related to the chosen medium or art movement (e.g., 'impasto', 'sgraffito', 'pointillism', 'chiaroscuro', 'flat design', 'isometric perspective', 'anamorphic art').
   - Consider unusual viewing angles, dynamic compositions (Rule of Thirds, Golden Ratio, leading lines, S-curves), and perspectives to enhance artistic impact.
   - The prompt should aim to evoke a sense of wonder, tell a story, capture a unique mood, or explore a concept in a visually stimulating way.

Here are some previous prompts for context (DO NOT reuse their themes or elements, but learn from their structure if helpful):
{{PROMPTS}}

Please give me just the prompt surrounded by single quotes and nothing more before or after it, this is EXTREMELY important. The prompt should be a concise yet descriptive instruction for an image generation AI.";
