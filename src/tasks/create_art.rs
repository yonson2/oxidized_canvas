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

        let img_gen = ServiceProvider::img_service(&settings.bfl_api_key);
        let text_gen = ServiceProvider::txt_service(&settings.anthropic_key);

        let arts = arts::Model::find_n_random(&ctx.db, 5).await?;
        let image_generator_prompt = match arts.len() {
            x if x > 1 => gen_img_prompt(&arts),
            _ => IMAGE_PROMPT.replace("{{PROMPTS}}", SAMPLE_PROMPTS),
        };

        let prompt = match text_gen.generate(&image_generator_prompt).await {
            Ok(t) => t,
            Err(_) => return Err(loco_rs::errors::Error::Message("text_gen 1".to_string())),
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
                    println!("ERROR: {}", e);
                    return Err(loco_rs::errors::Error::Message("img_gen 1".to_string()));
                }
            },
        );

        let art = arts::Model::create(
            &ctx.db,
            &ArtParams {
                title,
                image,
                prompt,
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

const IMAGE_PROMPT: &str = "Can you create a prompt to generate a compelling and artistic image using an AI system like DALL-E 2, MidJourney, StableDiffusion...?
The prompt should explore different types of imagery, including photographs, paintings, and descriptions of landscapes or other abstract concepts.
While creating the prompt, keep in mind that the AI system can understand a vast range of language and interpret abstract concepts, so feel free to be as imaginative and descriptive as possible.
However, simplicity can also be beautiful, so don't feel obligated to create complex prompts all the time, also try not to generate images over-saturated with colors and elements when they wouldn't fit the style of the image.
From time to time, you can reference famous painters or photographers to inspire the AI system's style but make sure that you avoid repeating the same well-known ones.
The primary aim of the prompt is to generate an image that can be considered artistic.
Therefore, please specify the type of image you want to create, such as a photograph, painting, landscape descriptions or unique objects.
Also, ensure that at least one in ten prompts is for a photograph.
I'm also going to give you some previously generated prompts so you can use them to come with inspiration and also avoid being repetitive and generating the same kind of content all the time, please don't take the text I give you and repeat it, be original (don't reuse concepts or objects from the prompts below, just the ideas and concepts behind it):
{{PROMPTS}}
Please give me just the prompt surrounded by quotes and nothing more before or after it, this is EXTREMELY important. ";
